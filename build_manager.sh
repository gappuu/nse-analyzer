#!/bin/bash

# NSE Options Analyzer - Build & Deploy Script
# Run this from the nse-analyzer directory (parent directory)

set -e

echo "🚀 NSE Options Analyzer - Build & Deploy Manager"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_header() { echo -e "${CYAN}[STEP]${NC} $1"; }

# Verify we're in the correct directory
if [ ! -d "backend" ] || [ ! -d "frontend" ] || [ ! -d "tauri-app" ]; then
    print_error "Please run this script from the nse-analyzer directory (parent directory)!"
    echo "Expected structure:"
    echo "nse-analyzer/           ← Run script from here"
    echo "├── backend/"
    echo "├── frontend/"
    echo "└── tauri-app/"
    exit 1
fi

# Function to check dependencies
check_dependencies() {
    print_header "Checking dependencies..."
    
    local all_deps_ok=true
    
    # Check Rust
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust found: $RUST_VERSION"
    else
        print_error "Rust not found!"
        echo "Install from: https://rustup.rs/"
        all_deps_ok=false
    fi
    
    # Check Node.js
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version)
        print_success "Node.js found: $NODE_VERSION"
    else
        print_error "Node.js not found!"
        echo "Install from: https://nodejs.org/"
        all_deps_ok=false
    fi
    
    # Check Tauri CLI
    if command -v cargo &> /dev/null && cargo install --list | grep -q "tauri-cli"; then
        print_success "Tauri CLI found"
    else
        print_warning "Tauri CLI not found!"
        echo "Install with: cargo install tauri-cli --version '^2.0.0'"
        all_deps_ok=false
    fi
    
    echo ""
    print_header "Checking build artifacts..."
    
    # Check backend binary
    if [ -f "backend/target/release/nse-analyzer" ]; then
        print_success "Backend binary exists: backend/target/release/nse-analyzer"
    else
        print_warning "Backend binary not found: backend/target/release/nse-analyzer"
    fi
    
    # Check frontend static files
    if [ -d "frontend/out" ]; then
        print_success "Frontend static files exist: frontend/out/"
    else
        print_warning "Frontend static files not found: frontend/out/"
    fi
    
    # Check resource binary
    if [ -f "tauri-app/src-tauri/resource/nse-analyzer" ]; then
        print_success "Resource binary exists: tauri-app/src-tauri/resource/nse-analyzer"
    else
        print_warning "Resource binary not found: tauri-app/src-tauri/resource/nse-analyzer"
    fi
    
    # Check icons
    ICON_DIR="tauri-app/src-tauri/icons"
    REQUIRED_ICONS=("32x32.png" "128x128.png" "128x128@2x.png" "icon.icns" "icon.ico")
    MISSING_ICONS=()
    
    for icon in "${REQUIRED_ICONS[@]}"; do
        if [ ! -f "$ICON_DIR/$icon" ]; then
            MISSING_ICONS+=("$icon")
        fi
    done
    
    if [ ${#MISSING_ICONS[@]} -eq 0 ]; then
        print_success "All required icons found in $ICON_DIR/"
    else
        print_warning "Missing icon files:"
        for icon in "${MISSING_ICONS[@]}"; do
            echo "  - $ICON_DIR/$icon"
        done
    fi
    
    echo ""
    if [ "$all_deps_ok" = false ]; then
        print_error "Please install missing dependencies before proceeding."
        return 1
    else
        print_success "All dependencies check complete!"
        return 0
    fi
}

# Function to clean builds
clean_builds() {
    print_header "Cleaning all builds..."
    
    # Clean backend
    print_status "Cleaning backend (cargo clean)..."
    cd backend
    cargo clean
    cd ..
    print_success "Backend cleaned"
    
    # Clean frontend
    print_status "Removing frontend/out directory..."
    if [ -d "frontend/out" ]; then
        rm -rf frontend/out
        print_success "Frontend out/ directory removed"
    else
        print_warning "Frontend out/ directory doesn't exist"
    fi
    
    # Clean tauri-app
    print_status "Cleaning tauri-app..."
    cd tauri-app/src-tauri
    cargo clean
    
    # Empty resource folder
    print_status "Emptying resource folder..."
    if [ -d "resource" ]; then
        rm -rf resource/*
        print_success "Resource folder emptied"
    else
        print_warning "Resource folder doesn't exist"
    fi
    cd ../..
    
    print_success "All builds cleaned successfully!"
}

# Function to build and deploy
build_and_deploy() {
    print_header "Building and deploying application..."
    
    # Build backend
    print_status "Building backend with release profile..."
    cd backend
    CARGO_INCREMENTAL=0 cargo build --release
    cd ..
    print_success "Backend built successfully"
    
    # Copy backend binary to resource
    print_status "Copying backend binary to tauri-app/src-tauri/resource/..."
    mkdir -p tauri-app/src-tauri/resource
    cp backend/target/release/nse-analyzer tauri-app/src-tauri/resource/
    chmod +x tauri-app/src-tauri/resource/nse-analyzer
    print_success "Backend binary copied to resource folder"
    
    # Build frontend
    print_status "Building frontend static files..."
    cd frontend
    npm run build
    cd ..
    print_success "Frontend built successfully"
    
    # Build tauri app
    print_status "Building Tauri application..."
    cd tauri-app
    npm run build
    cd ..
    print_success "Tauri application built successfully"
    
    echo ""
    print_success "Build and deploy complete!"
    echo ""
    echo "Output location:"
    echo "  tauri-app/src-tauri/target/release/bundle/"
    echo "  ├── dmg/     (macOS installer)"
    echo "  ├── msi/     (Windows installer)"
    echo "  └── deb/     (Linux installer)"
}

# Main menu
show_menu() {
    echo ""
    echo "========================================="
    echo "  NSE Analyzer - Build Management"
    echo "========================================="
    echo ""
    echo "1) Clean all builds"
    echo "2) Build & Deploy (locally)"
    echo "3) Check dependencies & artifacts"
    echo "4) Exit"
    echo ""
}

# Main loop
while true; do
    show_menu
    read -p "Select an option (1-4): " choice
    echo ""
    
    case $choice in
        1)
            clean_builds
            ;;
        2)
            build_and_deploy
            ;;
        3)
            check_dependencies
            ;;
        4)
            print_status "Exiting..."
            exit 0
            ;;
        *)
            print_error "Invalid option. Please select 1-4."
            ;;
    esac
    
    echo ""
    read -p "Press Enter to continue..."
done