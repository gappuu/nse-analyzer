#!/bin/bash

# NSE Options Analyzer - Tauri Setup Script (Organized Structure)
# Run this from the tauri-app directory

set -e

echo "🚀 NSE Options Analyzer - Tauri Setup (Organized Structure)"
echo "=========================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Verify we're in the right directory
if [ ! -f "src-tauri/tauri.conf.json" ]; then
    print_error "Please run this script from the tauri-app directory!"
    echo "Expected structure:"
    echo "nse-analyzer/"
    echo "├── backend/" 
    echo "├── frontend/"
    echo "└── tauri-app/     ← Run script from here"
    exit 1
fi

print_status "Verifying folder structure..."

# Check for backend
BACKEND_RESOURCE="src-tauri/resource/nse-analyzer"
BACKEND_BUILD="../backend/target/release/nse-analyzer"

# Check if resource binary exists and is non-empty
if [ -s "$BACKEND_RESOURCE" ]; then
    print_success "Backend binary found: $BACKEND_RESOURCE"

else
    print_warning "Backend binary missing or empty at $BACKEND_RESOURCE"

    # Check fallback build location
    if [ -s "$BACKEND_BUILD" ]; then
        print_success "Found compiled backend at $BACKEND_BUILD"
        echo "Copying backend binary to resources..."

        mkdir -p "$(dirname "$BACKEND_RESOURCE")"
        cp "$BACKEND_BUILD" "$BACKEND_RESOURCE"
        chmod +x "$BACKEND_RESOURCE"

        print_success "Backend binary copied to $BACKEND_RESOURCE"
    else
        print_warning "No compiled backend found at $BACKEND_BUILD"
        echo ""
        echo "Please compile your Rust backend first:"
        echo "  cd ../backend && CARGO_INCREMENTAL=0 cargo build --release"
        echo ""
        exit 1
    fi
fi

# Check for frontend
if [ ! -d "../frontend/out" ]; then
    print_error "Frontend Static directory not found at ../frontend/out"
    echo "from frontend folder-> run command "npm run build-static""
    exit 1
else
    print_success "Frontend directory found: ../frontend"
fi

# Check prerequisites
print_status "Checking prerequisites..."

# Check Node.js
if ! command -v node &> /dev/null; then
    print_error "Node.js not found! Please install Node.js 18+ first."
    echo "Visit: https://nodejs.org/"
    exit 1
else
    NODE_VERSION=$(node --version)
    print_success "Node.js found: $NODE_VERSION"
fi

# Check Rust
if ! command -v rustc &> /dev/null; then
    print_warning "Rust not found! Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed successfully"
else
    RUST_VERSION=$(rustc --version)
    print_success "Rust found: $RUST_VERSION"
fi

# Check Tauri CLI
if ! command -v tauri &> /dev/null; then
    print_warning "Tauri CLI not found! Installing..."
    cargo install tauri-cli --version "^2.0.0"
    print_success "Tauri CLI installed successfully"
else
    print_success "Tauri CLI found"
fi

# Install Tauri dependencies
print_status "Installing Tauri dependencies..."
npm install
print_success "Tauri dependencies installed"

# Check frontend dependencies
print_status "Checking frontend dependencies..."
if [ ! -d "../frontend/node_modules" ]; then
    print_warning "Frontend dependencies not found. Installing..."
    cd ../frontend
    npm install
    cd ../tauri-app
    print_success "Frontend dependencies installed"
else
    print_success "Frontend dependencies found"
fi

# Check for icons
print_status "Checking app icons..."
ICON_DIR="src-tauri/icons"
REQUIRED_ICONS=("32x32.png" "128x128.png" "128x128@2x.png" "icon.icns" "icon.ico")
MISSING_ICONS=()

for icon in "${REQUIRED_ICONS[@]}"; do
    if [ ! -f "$ICON_DIR/$icon" ]; then
        MISSING_ICONS+=("$icon")
    fi
done

if [ ${#MISSING_ICONS[@]} -gt 0 ]; then
    print_warning "Missing icon files:"
    for icon in "${MISSING_ICONS[@]}"; do
        echo "  - $ICON_DIR/$icon"
    done
    echo ""
    echo "See src-tauri/icons/README.md for icon generation instructions."
    echo ""
else
    print_success "All required icons found"
fi

# Platform-specific info
print_status "Platform detection..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_success "macOS detected - ready for .dmg creation"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_success "Linux detected - ready for .deb creation"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    print_success "Windows detected - ready for .msi creation"
fi

echo ""
print_success "Setup complete! Your folder structure:"
echo ""
echo "nse-analyzer/"
echo "├── backend/"
echo "│   └── target/release/nse-analyzer    ← Your Rust backend"
echo "├── frontend/"
echo "│   ├── app/                          ← Next.js frontend"
echo "│   └── out/                          ← Built static files"
echo "└── tauri-app/"
echo "    ├── src-tauri/                            ← Tauri configuration"
echo "    └── package.json                          ← Tauri dependencies"
echo "    ├── src-tauri/resource/nse-analyzer       ← Backend binary copied here"
echo ""
print_status "Next steps:"
echo ""
echo "1. Build desktop app:"
echo "   npm run build                  # Creates .dmg/.msi/.deb"
echo ""
echo "2. Development commands (run from tauri-app/):"
echo "   npm run dev                    # Full Tauri development"
echo "   npm run frontend:dev           # Frontend development only"
echo ""
echo "Output will be in:"
echo "  tauri-app/src-tauri/target/release/bundle/"
echo "  ├── dmg/     (macOS installer)"
echo "  ├── msi/     (Windows installer)"
echo "  └── deb/     (Linux installer)"