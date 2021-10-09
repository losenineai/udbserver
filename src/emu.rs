use crate::arch;
use crate::capi::uc_hook;
use crate::reg::RegMap;
use crate::DynResult;

use gdbstub::target;
use gdbstub::target::ext::base::singlethread::{GdbInterrupt, ResumeAction, SingleThreadOps, StopReason};
use gdbstub::target::ext::base::SendRegisterOutput;
use gdbstub::target::ext::breakpoints::WatchKind;
use gdbstub::target::{Target, TargetError, TargetResult};
use std::collections::HashMap;
use unicorn::unicorn_const::uc_error;
use unicorn::unicorn_const::{HookType, MemType, Mode, Query};
use unicorn::UnicornHandle;

struct EmuState {
    step_state: bool,
    step_hook: Option<uc_hook>,
    watch_addr: Option<u64>,
}

static mut G: EmuState = EmuState {
    step_state: false,
    step_hook: None,
    watch_addr: None,
};

fn step_hook(mut uc: UnicornHandle, _addr: u64, _size: u32) {
    let mut addr = None;
    unsafe {
        if G.step_state {
            G.step_state = false;
            return;
        }
        if let Some(step_hook) = G.step_hook {
            uc.remove_hook(step_hook).expect("Failed to remove step hook");
            G.step_hook = None;
        }
        if let Some(watch_addr) = G.watch_addr {
            addr = Some(watch_addr);
            G.watch_addr = None
        }
    }
    crate::udbserver_resume(addr).expect("Failed to resume udbserver");
}

fn mem_hook(mut uc: UnicornHandle, _mem_type: MemType, addr: u64, _size: usize, _value: i64) {
    unsafe {
        if G.watch_addr == None {
            G.watch_addr = Some(addr);
            if G.step_hook.is_none() {
                G.step_hook = Some(uc.add_code_hook(1, 0, step_hook).expect("Failed to add code hook"));
            }
        }
    }
}

pub struct Emu {
    uc: UnicornHandle<'static>,
    reg_map: &'static RegMap,
    bp_sw_hooks: HashMap<u64, uc_hook>,
    bp_hw_hooks: HashMap<u64, uc_hook>,
    wp_r_hooks: HashMap<u64, HashMap<u64, uc_hook>>,
    wp_w_hooks: HashMap<u64, HashMap<u64, uc_hook>>,
    wp_rw_hooks: HashMap<u64, HashMap<u64, uc_hook>>,
}

impl Emu {
    pub fn new(uc: UnicornHandle<'static>) -> DynResult<Emu> {
        let arch = uc.get_arch();
        let query_mode = uc.query(Query::MODE).expect("Failed to query mode");
        let mode = Mode::from_bits(query_mode as i32).unwrap();
        let reg_map = RegMap::new(arch, mode);
        Ok(Emu {
            uc: uc,
            reg_map: reg_map,
            bp_sw_hooks: HashMap::new(),
            bp_hw_hooks: HashMap::new(),
            wp_r_hooks: HashMap::new(),
            wp_w_hooks: HashMap::new(),
            wp_rw_hooks: HashMap::new(),
        })
    }
}

impl Target for Emu {
    type Arch = arch::GenericArch;
    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn breakpoints(&mut self) -> Option<target::ext::breakpoints::BreakpointsOps<Self>> {
        Some(self)
    }

    #[inline(always)]
    fn target_description_xml_override(&mut self) -> Option<target::ext::target_description_xml_override::TargetDescriptionXmlOverrideOps<Self>> {
        Some(self)
    }
}

impl SingleThreadOps for Emu {
    fn resume(&mut self, action: ResumeAction, _gdb_interrupt: GdbInterrupt<'_>) -> Result<Option<StopReason<u64>>, Self::Error> {
        match action {
            ResumeAction::Step => {
                unsafe {
                    G.step_state = true;
                    G.step_hook = Some(self.uc.add_code_hook(1, 0, step_hook).map_err(|_| "Failed to add code hook")?);
                }
                Ok(None)
            }
            ResumeAction::Continue => Ok(None),
            _ => Err("Cannot resume with signal"),
        }
    }

    fn read_registers(&mut self, regs: &mut arch::GenericRegs) -> TargetResult<(), Self> {
        regs.buf = Vec::new();
        for reg in self.reg_map.reg_list() {
            let val = match reg.0 {
                Some(regid) => self.uc.reg_read(regid).map_err(|_| ())?,
                None => 0,
            };
            regs.buf.extend(self.reg_map.to_bytes(val, reg.1));
        }
        Ok(())
    }

    fn write_registers(&mut self, regs: &arch::GenericRegs) -> TargetResult<(), Self> {
        let mut i = 0;
        for reg in self.reg_map.reg_list() {
            let part = &regs.buf[i..i + reg.1];
            let val = self.reg_map.from_bytes(part);
            i += reg.1;
            if let Some(regid) = reg.0 {
                self.uc.reg_write(regid, val).map_err(|_| ())?
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn single_register_access(&mut self) -> Option<target::ext::base::SingleRegisterAccessOps<(), Self>> {
        Some(self)
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<(), Self> {
        match self.uc.mem_read(start_addr as u64, data) {
            Ok(_) => Ok(()),
            Err(uc_error::READ_UNMAPPED) => Err(TargetError::Errno(1)),
            Err(_) => Err(TargetError::Fatal("Failed to read addr")),
        }
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        match self.uc.mem_write(start_addr as u64, data) {
            Ok(_) => Ok(()),
            Err(uc_error::WRITE_UNMAPPED) => Err(TargetError::Errno(1)),
            Err(_) => Err(TargetError::Fatal("Failed to write addr")),
        }
    }
}

impl target::ext::breakpoints::Breakpoints for Emu {
    #[inline(always)]
    fn sw_breakpoint(&mut self) -> Option<target::ext::breakpoints::SwBreakpointOps<Self>> {
        Some(self)
    }

    #[inline(always)]
    fn hw_breakpoint(&mut self) -> Option<target::ext::breakpoints::HwBreakpointOps<Self>> {
        Some(self)
    }

    #[inline(always)]
    fn hw_watchpoint(&mut self) -> Option<target::ext::breakpoints::HwWatchpointOps<Self>> {
        Some(self)
    }
}

macro_rules! add_breakpoint {
    ( $self:ident, $addr:ident, $hook_map:ident ) => {{
        let hook = match $self.uc.add_code_hook($addr.into(), $addr.into(), step_hook) {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        $self.$hook_map.insert($addr.into(), hook);
        Ok(true)
    }};
    ( $self:ident, $mem_type:ident, $addr:ident, $len:ident, $hook_map:ident ) => {{
        let hook = match $self.uc.add_mem_hook(HookType::$mem_type, $addr.into(), ($addr + $len - 1).into(), mem_hook) {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        $self.$hook_map.entry($len).or_insert(HashMap::new()).insert($addr.into(), hook);
        Ok(true)
    }};
}

macro_rules! remove_breakpoint {
    ( $self:ident, $addr:ident, $hook_map:ident ) => {{
        let hook = match $self.$hook_map.remove(&$addr.into()) {
            Some(h) => h,
            None => return Ok(false),
        };
        match $self.uc.remove_hook(hook) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }};
    ( $self:ident, $addr:ident, $len:ident, $hook_map:ident ) => {{
        let map = match $self.$hook_map.get_mut(&$len) {
            Some(h) => h,
            None => return Ok(false),
        };
        let hook = match map.remove(&$addr.into()) {
            Some(h) => h,
            None => return Ok(false),
        };
        match $self.uc.remove_hook(hook) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }};
}

impl target::ext::breakpoints::SwBreakpoint for Emu {
    fn add_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        add_breakpoint!(self, addr, bp_sw_hooks)
    }

    fn remove_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        remove_breakpoint!(self, addr, bp_sw_hooks)
    }
}

impl target::ext::breakpoints::HwBreakpoint for Emu {
    fn add_hw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        add_breakpoint!(self, addr, bp_hw_hooks)
    }

    fn remove_hw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        remove_breakpoint!(self, addr, bp_hw_hooks)
    }
}

impl target::ext::breakpoints::HwWatchpoint for Emu {
    fn add_hw_watchpoint(&mut self, addr: u64, len: u64, kind: WatchKind) -> TargetResult<bool, Self> {
        match kind {
            WatchKind::Read => add_breakpoint!(self, MEM_READ, addr, len, wp_r_hooks),
            WatchKind::Write => add_breakpoint!(self, MEM_WRITE, addr, len, wp_w_hooks),
            WatchKind::ReadWrite => add_breakpoint!(self, MEM_VALID, addr, len, wp_rw_hooks),
        }
    }

    fn remove_hw_watchpoint(&mut self, addr: u64, len: u64, kind: WatchKind) -> TargetResult<bool, Self> {
        match kind {
            WatchKind::Read => remove_breakpoint!(self, addr, len, wp_r_hooks),
            WatchKind::Write => remove_breakpoint!(self, addr, len, wp_w_hooks),
            WatchKind::ReadWrite => remove_breakpoint!(self, addr, len, wp_rw_hooks),
        }
    }
}

impl target::ext::base::SingleRegisterAccess<()> for Emu {
    fn read_register(&mut self, _tid: (), reg_id: arch::GenericRegId, mut output: SendRegisterOutput<'_>) -> TargetResult<(), Self> {
        let reg = self.reg_map.get_reg(reg_id.0)?;
        if reg.1 <= 8 {
            let val = match reg.0 {
                Some(regid) => self.uc.reg_read(regid).map_err(|_| ())?,
                None => 0,
            };
            output.write(&self.reg_map.to_bytes(val, reg.1));
        } else {
            if let Some(regid) = reg.0 {
                output.write(&self.uc.reg_read_long(regid).map_err(|_| ())?);
            }
        }
        Ok(())
    }

    fn write_register(&mut self, _tid: (), reg_id: arch::GenericRegId, val: &[u8]) -> TargetResult<(), Self> {
        let reg = self.reg_map.get_reg(reg_id.0)?;
        assert!(reg.1 == val.len(), "Length mismatch when write register {}", reg.0.unwrap());
        if let Some(regid) = reg.0 {
            if reg.1 <= 8 {
                let v = self.reg_map.from_bytes(val);
                self.uc.reg_write(regid, v).map_err(|_| ())?;
            } else {
                self.uc.reg_write_long(regid, val.into()).map_err(|_| ())?;
            }
        }
        Ok(())
    }
}

impl target::ext::target_description_xml_override::TargetDescriptionXmlOverride for Emu {
    fn target_description_xml(&self) -> &str {
        self.reg_map.description_xml()
    }
}
