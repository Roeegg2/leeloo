const std = @import("std");
const Build = std.Build;

// Project layout.
const kernel_root = "src/main.zig";
const kernel_entry = "kmain"; // must match ENTRY() in linker.ld
const linker_script = "linker.ld";
const limine_conf = "limine.conf";
const make_iso_script = "scripts/make-iso.sh";

// QEMU flags shared by every run/debug step. The ISO is appended after `-cdrom`.
const qemu_base_args = [_][]const u8{
    "qemu-system-x86_64",
    "-m",
    "1G",
    "-serial",
    "stdio",
    "-boot",
    "d",
    "-no-reboot",
    "-no-shutdown",
};

pub fn build(b: *Build) void {
    const target = kernelTarget(b);
    const optimize = b.standardOptimizeOption(.{});

    const limine_zig = b.dependency("limine_zig", .{
        // The API revision of the Limine Boot Protocol to use. Defaults to 0.
        .api_revision = 3,
        // If false, the build fails if deprecated features are used.
        .allow_deprecated = false,
        // If true, pointer fields are exposed as raw addresses.
        .no_pointers = false,
    });
    const limine_module = limine_zig.module("limine");

    // Kernel (honours -Doptimize) -> installed into zig-out/
    const kernel = addKernel(b, "kernel", target, optimize, limine_module);
    b.installArtifact(kernel);

    const run = addQemu(b, addIso(b, kernel, "leeloo.iso"), &.{});
    b.step("run", "Run the kernel in QEMU").dependOn(&run.step);

    const debug_kernel = addKernel(b, "kernel-debug", target, .Debug, limine_module);
    const install_debug = b.addInstallArtifact(debug_kernel, .{});
    const debug_iso = addIso(b, debug_kernel, "leeloo-debug.iso");

    const debug = addQemu(b, debug_iso, &.{
        "-s",
        "-S",
        "-d",
        "int,cpu_reset,guest_errors",
        "-D",
        "qemu.log",
    });
    debug.step.dependOn(&install_debug.step);
    b.step("debug", "Run the debug kernel in QEMU, paused for GDB").dependOn(&debug.step);

    // Build only, no QEMU. Terminates, so it's safe to call from nvim synchronously.
    const build_debug = b.step("build-debug", "Build debug kernel + ISO without running QEMU");
    build_debug.dependOn(&install_debug.step);
    build_debug.dependOn(&b.addInstallFile(debug_iso, "leeloo-debug.iso").step);

    const clean = b.addSystemCommand(&.{ "rm", "-rf", "zig-out", ".zig-cache" });
    b.step("clean", "Remove generated build artifacts").dependOn(&clean.step);
}

/// Freestanding x86_64 with all SIMD/FPU state disabled, so the kernel never
/// has to save or restore it. Floating point falls back to software.
fn kernelTarget(b: *Build) Build.ResolvedTarget {
    const x86 = std.Target.x86;
    return b.resolveTargetQuery(.{
        .cpu_arch = .x86_64,
        .os_tag = .freestanding,
        .abi = .none,
        .cpu_features_add = x86.featureSet(&.{.soft_float}),
        .cpu_features_sub = x86.featureSet(&.{ .avx, .avx2, .sse, .sse2, .mmx }),
    });
}

fn addKernel(
    b: *Build,
    name: []const u8,
    target: Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    limine: *Build.Module,
) *Build.Step.Compile {
    const exe = b.addExecutable(.{
        .name = name,
        .root_module = b.createModule(.{
            .root_source_file = b.path(kernel_root),
            .target = target,
            .optimize = optimize,
            .code_model = .kernel,
            // to prevent interrupts from clobbering the redzone
            .red_zone = false,
            .imports = &.{
                .{ .name = "limine", .module = limine },
            },
        }),
        .use_llvm = true,
    });
    exe.setLinkerScript(b.path(linker_script));
    // Zig passes `--entry _start` to LLD by default, which overrides ENTRY() in the
    // linker script. Set it explicitly so the ELF gets the right entry point.
    exe.entry = .{ .symbol_name = kernel_entry };
    return exe;
}

/// Bundles the kernel and Limine config into a bootable ISO.
fn addIso(b: *Build, kernel: *Build.Step.Compile, iso_name: []const u8) Build.LazyPath {
    const cmd = b.addSystemCommand(&.{ "sh", make_iso_script });
    cmd.addFileArg(kernel.getEmittedBin());
    cmd.addFileArg(b.path(limine_conf));
    return cmd.addOutputFileArg(iso_name);
}

/// Boots `iso` in QEMU. The ISO dependency is tracked automatically via the LazyPath.
fn addQemu(b: *Build, iso: Build.LazyPath, extra_args: []const []const u8) *Build.Step.Run {
    const cmd = b.addSystemCommand(&qemu_base_args);
    cmd.addArg("-cdrom");
    cmd.addFileArg(iso);
    cmd.addArgs(extra_args);
    return cmd;
}
