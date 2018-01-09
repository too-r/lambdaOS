use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};
use super::disable_interrupts_and_then;

pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut ExceptionStackFrame) {
    disable_interrupts_and_then(|| {
        println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
        loop {}
    });
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!(
        "\nEXCEPTION: BREAKPOINT at {:#x}\n{:#?}",
        stack_frame.instruction_pointer,
        stack_frame
    );
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut ExceptionStackFrame) {
    disable_interrupts_and_then(|| {
        println!(
            "\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
            stack_frame.instruction_pointer,
            stack_frame
        );
        loop {}
    });
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    error_code: PageFaultErrorCode,
) {
    disable_interrupts_and_then(|| {
        use x86_64::registers::control_regs;
        println!(
            "\nEXCEPTION: PAGE FAULT while accessing {:#x}\nerror code: \
             {:?}\n{:#?}",
            control_regs::cr2(),
            error_code,
            stack_frame
        );
        loop {}
    });
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    disable_interrupts_and_then(|| {
        println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
        loop {}
    });
}

pub extern "x86-interrupt" fn gpf_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
)
{
    disable_interrupts_and_then(|| {
        println!("\nEXCEPTION: GPF\n{:#?}", stack_frame);
        loop {}
    });
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
)
{
    disable_interrupts_and_then(|| {
        println!("\nEXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
        loop {}
    });
}
