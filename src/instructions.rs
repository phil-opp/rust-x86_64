//! Low level functions for special x86 instructions.

use segmentation;

/// Enable hardware interrupts using the `sti` instruction.
pub unsafe fn enable_interrupts() {
    asm!("sti");
}

/// Disable hardware interrupts using the `cli` instruction.
pub unsafe fn disable_interrupts() {
    asm!("cli");
}

/// Generate a software interrupt.
/// This is a macro because the argument needs to be an immediate.
#[macro_export]
macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: u64,
}

/// Load GDT table.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table.
pub unsafe fn lldt(ldt: &DescriptorTablePointer) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

/// Load the task state register.
pub unsafe fn load_tss(sel: segmentation::SegmentSelector) {
    asm!("ltr $0" :: "r" (sel.bits()));
}

/// Halts the CPU by executing the `hlt` instruction.
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}

/// Read time stamp counters

/// Read the time stamp counter using the `RDTSC` instruction.
///
/// The `RDTSC` instruction is not a serializing instruction.
/// It does not necessarily wait until all previous instructions
/// have been executed before reading the counter. Similarly,
/// subsequent instructions may begin execution before the
/// read operation is performed. If software requires `RDTSC` to be
/// executed only after all previous instructions have completed locally,
/// it can either use `RDTSCP` or execute the sequence `LFENCE;RDTSC`.
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtsc" : "={eax}" (low), "={edx}" (high));
    }
    ((u64::from(high)) << 32) | (u64::from(low))
}

/// Read the time stamp counter using the `RDTSCP` instruction.
///
/// The `RDTSCP` instruction waits until all previous instructions
/// have been executed before reading the counter.
/// However, subsequent instructions may begin execution
/// before the read operation is performed.
///
/// Volatile is used here because the function may be used to act as
/// an instruction barrier.
pub fn rdtscp() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtscp" : "={eax}" (low), "={edx}" (high) ::: "volatile");
    }
    ((high as u64) << 32) | (low as u64)
}

// Model specific registers

/// Write 64 bits to msr register.
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!("wrmsr" :: "{ecx}" (msr), "{eax}" (low), "{edx}" (high) : "memory" : "volatile" );
}

/// Read 64 bits msr register.
pub fn rdmsr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        asm!("rdmsr" : "={eax}" (low), "={edx}" (high) : "{ecx}" (msr) : "memory" : "volatile");
    }
    ((high as u64) << 32) | (low as u64)
}

/// I/O port functionality.
pub mod port {
    /// Write 8 bits to I/O port.
    pub unsafe fn outb(port: u16, val: u8) {
        asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(val));
    }

    /// Read 8 bits from I/O port.
    pub unsafe fn inb(port: u16) -> u8 {
        let ret: u8;
        asm!("inb %dx, %al" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
        ret
    }

    /// Write 16 bits to I/O port.
    pub unsafe fn outw(port: u16, val: u16) {
        asm!("outw %ax, %dx" :: "{dx}"(port), "{al}"(val));
    }

    /// Read 16 bits from I/O port.
    pub unsafe fn inw(port: u16) -> u16 {
        let ret: u16;
        asm!("inw %dx, %ax" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
        ret
    }

    /// Write 32 bits to I/O port.
    pub unsafe fn outl(port: u16, val: u32) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{al}"(val));
    }

    /// Read 32 bits from I/O port.
    pub unsafe fn inl(port: u16) -> u32 {
        let ret: u32;
        asm!("inl %dx, %eax" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
        ret
    }

    /// Write 8-bit array to I/O port.
    pub unsafe fn outsb(port: u16, buf: &[u8]) {
        asm!("rep outsb (%esi), %dx"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
            : "ecx", "edi");
    }

    /// Read 8-bit array from I/O port.
    pub unsafe fn insb(port: u16, buf: &mut [u8]) {
        asm!("rep insb %dx, (%edi)"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
            : "ecx", "edi" : "volatile");
    }

    /// Write 16-bit array to I/O port.
    pub unsafe fn outsw(port: u16, buf: &[u16]) {
        asm!("rep outsw (%esi), %dx"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
            : "ecx", "edi");
    }

    /// Read 16-bit array from I/O port.
    pub unsafe fn insw(port: u16, buf: &mut [u16]) {
        asm!("rep insw %dx, (%edi)"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
            : "ecx", "edi" : "volatile");
    }

    /// Write 32-bit array to I/O port.
    pub unsafe fn outsl(port: u16, buf: &[u32]) {
        asm!("rep outsl (%esi), %dx"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
            : "ecx", "edi");
    }

    /// Read 32-bit array from I/O port.
    pub unsafe fn insl(port: u16, buf: &mut [u32]) {
        asm!("rep insl %dx, (%edi)"
            :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
            : "ecx", "edi" : "volatile");
    }
}