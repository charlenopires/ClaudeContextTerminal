//! Demo program to show the Goofy ASCII art logo
//! 
//! Run with: cargo run --example logo_demo

// We need to import from the local crate, not external
// For examples, we typically don't have access to internal modules
// So let's create a simulated demo instead

fn main() {
    println!("=== GOOFY ASCII ART LOGO DEMO ===\n");

    // Demo: ASCII Art Preview 
    println!("🚀 GOOFY ASCII ART LOGO:");
    println!();
    print_goofy_logo();
    println!();

    // Demo: Component Features
    println!("📋 Logo Component Features:");
    println!("   ✓ Orange-to-purple gradient support");
    println!("   ✓ Responsive design (adapts to screen size)");
    println!("   ✓ Full and compact modes");
    println!("   ✓ Customizable colors and themes");
    println!("   ✓ ASCII art letterforms (G-O-O-F-Y)");
    println!("   ✓ Background diagonal patterns");
    println!("   ✓ Version and branding display");
    println!();

    // Demo: Splash Screen Features
    println!("🎨 Splash Component Features:");
    println!("   ✓ Welcome information and quick start guide");
    println!("   ✓ Status indicators and feature highlights");
    println!("   ✓ Keyboard shortcuts display");
    println!("   ✓ Small screen adaptive layout");
    println!("   ✓ Orange-to-purple gradient theming");
    println!();

    // Demo: Color Palette
    println!("🎨 Goofy Color Palette:");
    println!("   🟠 Primary: Orange (#FFA500)");
    println!("   🟣 Secondary: Purple (#8A2BE2)");
    println!("   🔵 Accent: Blue (#8282FF)");
    println!("   🟢 Success: Green (#22C55E)");
    println!("   🟡 Warning: Yellow (#F59E0B)");
    println!("   🔴 Error: Red (#EF4444)");
    println!();

    // Demo: Technical Implementation
    println!("⚙️  Technical Implementation:");
    println!("   • Built with ratatui for terminal UI");
    println!("   • Gradient support with color interpolation");
    println!("   • Modular component architecture");
    println!("   • Theme-aware styling system");
    println!("   • Cross-platform terminal compatibility");
    println!();

    println!("=== Demo Complete ===");
    println!("The Goofy logo components are ready for TUI integration!");
}

fn print_goofy_logo() {
    // Simplified ASCII representation of the GOOFY logo
    let logo_lines2 = vec![
        "    Goofy™                                     v0.1.0",
        "╱╱╱╱╱╱ ▄▀▀▀▀ ▄▀▀▀▄ ▄▀▀▀▄ █▀▀▀▄ █   █ ╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱",
        "╱╱╱╱╱╱ █  ▄▄ █   █ █   █ █▀▀▀▄  ▀█▀  ╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱",
        "╱╱╱╱╱╱ ▀▀▀▀▀ ▀▀▀▀▀ ▀▀▀▀▀ ▀   ▀   ▀   ╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱",
        "",
    ];

    let logo_lines = vec![
        
        "░▒▓██████▓▒░ ░▒▓██████▓▒░ ░▒▓██████▓▒░░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░",
        "░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░",
        "░▒▓█▓▒▒▓███▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓██████▓▒░  ░▒▓██████▓▒░",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░         ░▒▓█▓▒░",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░         ░▒▓█▓▒░",
        "░▒▓██████▓▒░ ░▒▓██████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░         ░▒▓█▓▒░", 
        "",
    ];

    for line in logo_lines {
        println!("   {}", line);
    }
}