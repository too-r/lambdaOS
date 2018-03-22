#![allow(unused_imports)]
use x86_64::registers::msr::{rdmsr, wrmsr, IA32_APIC_BASE};
use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};
use acpi::madt::{IO_APICS, ISOS, NMIS, LOCAL_APICS};

lazy_static! {
    static ref BASE: AtomicU32 = {
        // Calculate base address.
        let address = rdmsr(IA32_APIC_BASE) & 0xffff0000;
        AtomicU32::new(address as u32)
    };
}

/// Interface to a local APIC.
pub struct LocalApic;

impl LocalApic {
    /// Read from a register of the Local APIC.
    pub fn lapic_read(which_reg: u32) -> u32 {
        let base = BASE.load(Ordering::SeqCst) as u32;
        unsafe { ptr::read_volatile(&(base as u32 + which_reg) as *const u32) }
    }

    /// Write to a register of the Local APIC.
    pub fn lapic_write(which_reg: u32, value: u32) {
        let base = BASE.load(Ordering::SeqCst) as u32;
        unsafe { ptr::write_volatile(&mut (base + which_reg) as *mut u32, value) };
    }

    pub fn lapic_set_nmi(vector: u8, _processor_id: u8, flags: u16, lint: u8) {
        // Set as NMI.
        let mut nmi: u32 = (800 | vector) as u32;
        // Active low.
        if flags & 2 == 0 {
            nmi |= 1 << 13;
        }

        // Level triggered.
        if flags & 8  == 0 {
            nmi |= 1 << 15;
        }

        match lint {
            1 => {
                LocalApic::lapic_write(0x360, nmi);
            },
            0 => {
                LocalApic::lapic_write(0x350, nmi);
            }
            _ => {},
        }
    }

    pub fn install_nmis() {
        for (i, nmi) in NMIS.lock().iter().enumerate() {
            LocalApic::lapic_set_nmi(0x90 + i as u8, nmi.processor_id, nmi.flags, nmi.lint_no);
        }
    }

    pub fn enable() {
        LocalApic::lapic_write(0xf0, LocalApic::lapic_read(0xf0) | 0x1ff);
    }
}

pub struct IoApic {
    pub id: u8,
    _resv: u8,
    pub address: u32,
    pub gsib: u32,
}

impl IoApic {
    pub fn install_redirects() {        
        // Install IRQ0, IRQ1.
        for iso in ISOS.lock().iter() {
            if iso.irq_source == 0 {
                IoApic::set_redirect(iso.irq_source, iso.gsi, iso.flags, LOCAL_APICS.lock()[0].id);
                break;
            } else if iso.irq_source == 1 {
                IoApic::set_redirect(iso.irq_source, iso.gsi, iso.flags, LOCAL_APICS.lock()[0].id);
                break;
            } else {
                break;
            }
        }
    }


    /// Get the I/O APIC that handles this GSI.
    pub fn io_apic_from_gsi(gsi: u32) -> Option<usize> {
        for apic in 0..IO_APICS.lock().len() {
            if IO_APICS.lock()[apic].gsib < gsi
                && IO_APICS.lock()[apic].gsib + IoApic::get_max_redirect(apic) > gsi
            {
                return Some(apic);
            } else {
                continue;
            }
        }

        None
    }

    /// Set the redirect for a given IRQ.
    #[allow(exceeding_bitshifts)]
    pub fn set_redirect(irq: u8, gsi: u32, flags: u16, id: u8) {
        let apic = IoApic::io_apic_from_gsi(gsi);

        if apic.is_none() {
            println!(
                "[ ERROR ] I/O APIC: Failed to find redirect for IRQ: {}",
                irq
            );
            return;
        } else {
            let io_apic = apic.unwrap();

            // Map IRQS: INT48 .. INT64
            let mut redirection: u64 = irq as u64 + 0x30;

            if flags & 2 == 0 {
                redirection |= 1 << 13;
            } else if flags & 8 == 0 {
                redirection |= 1 << 15;
            }

            redirection |= (id as u64) << 56;

            let ioredtbl: u32 = (gsi - IO_APICS.lock()[io_apic].gsib) * 2 + 16;

            IoApic::write(ioredtbl + 0, io_apic, redirection as u32);
            IoApic::write(ioredtbl + 1, io_apic, redirection as u32 >> 32);
        }
    }

    pub fn read(reg: u32, io_apic_num: usize) -> u32 {
        let mut base = IO_APICS.lock()[io_apic_num].address;
        unsafe {
            // Tell IOSEGSEL what register we want to use.
            let val = reg;
            let io_seg_sel = &mut base as *mut u32;
            ptr::write_volatile(io_seg_sel, val);

            // Read back from IOREGWIN.
            let io_seg_win = &mut (base + 4) as *mut u32;
            ptr::read_volatile(io_seg_win)
        }
    }

    pub fn write(reg: u32, io_apic_num: usize, data: u32) {
        let mut base = IO_APICS.lock()[io_apic_num].address;
        unsafe {
            let val = reg;
            let io_seg_sel = &mut base as *mut u32;
            ptr::write_volatile(io_seg_sel, val);

            let io_seg_win = &mut (base + 4) as *mut u32;
            ptr::write_volatile(io_seg_win, data);
        }
    }

    pub fn get_max_redirect(io_apic_num: usize) -> u32 {
        (IoApic::read(1, io_apic_num) & 0xff0000) >> 16
    }
}


pub fn init() {
    IoApic::install_redirects();
    LocalApic::install_nmis();
    LocalApic::enable();
}
