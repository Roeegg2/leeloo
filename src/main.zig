const std = @import("std");
const limine = @import("limine");
const arch = @import("x86_64/root.zig");
const serial = @import("x86_64/serial.zig");

export var start_marker: limine.RequestsStartMarker linksection(".limine_requests_start") = .{};
export var end_marker: limine.RequestsEndMarker linksection(".limine_requests_end") = .{};

export var base_revision: limine.BaseRevision linksection(".limine_requests") = .init(3);
export var framebuffer_request: limine.FramebufferRequest linksection(".limine_requests") = .{};

export fn kmain() noreturn {
    if (!base_revision.isSupported()) {
        @panic("Base revision not supported");
    }

    if (framebuffer_request.response) |framebuffer_response| {
        const framebuffer = framebuffer_response.getFramebuffers()[0];
        for (0..100) |i| {
            const fb_ptr: [*]volatile u32 = @ptrCast(@alignCast(framebuffer.address));
            fb_ptr[i * (framebuffer.pitch / 4) + i] = 0xffffff;
        }
    } else {
        @panic("Framebuffer response not present");
    }

    arch.init();

    serial.kprint("HALLO\n", .{});

    arch.hcf();
}

pub fn panic(msg: []const u8, error_return_trace: ?*std.builtin.StackTrace, ret_addr: ?usize) noreturn {
    _ = error_return_trace;
    _ = ret_addr;

    // make sure we can panic without being disturbed
    asm volatile ("cli");

    serial.kprint("KERNEL PANIC: {s}\n", .{msg});

    // "showdowns" are a better way than "meltdowns" for dealing with panic because that way we don't damage anything!
    arch.hcf();
}

