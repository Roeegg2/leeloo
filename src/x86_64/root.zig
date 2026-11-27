const builtin = @import("builtin");
const gdt = @import("gdt.zig");
const serial = @import("serial.zig");
const idt = @import("idt.zig");

pub fn init() void {
    serial.init() catch unreachable;
    gdt.load();
    idt.init();
}

// halt forever
pub fn hcf() noreturn {
    while (true) {
        asm volatile ("hlt");
    }
}
