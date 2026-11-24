const limine = @import("limine.zig");

export var base_revision: limine.BaseRevision linksection(".limine_requests") = .{ .revision = 3 };

fn outb(port: u16, value: u8) void {
    asm volatile ("outb %[v], %[p]"
        :
        : [v] "{al}" (value),
          [p] "{dx}" (port),
    );
}

fn serialPrint(s: []const u8) void {
    for (s) |c| outb(0x3F8, c);
}

export fn kmain() callconv(.c) noreturn {
    serialPrint("hello from leeloo kmainnnnnnnnnnnn\n");
    while (true) asm volatile ("hlt");
}
