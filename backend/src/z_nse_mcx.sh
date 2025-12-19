#!/bin/bash
# Example scripts for running the NSE-MCX analyzer with separate servers

echo "NSE-MCX Analyzer Example Scripts (Separate Servers)"
echo "===================================================="
echo ""

echo "🔧 DEVELOPMENT ORCHESTRATION:"
echo "# Use the orchestration script for easy management"
echo "./orchestrate.sh start           # Start both servers"
echo "./orchestrate.sh stop            # Stop both servers"
echo "./orchestrate.sh status          # Check status"
echo "./orchestrate.sh logs            # View logs"
echo ""

echo "🚀 MANUAL SERVER STARTUP:"
echo "1. Start NSE Server:"
echo "MODE=server EXCHANGE=nse PORT=3001 cargo run"
echo ""

echo "2. Start MCX Server (in separate terminal):"
echo "MODE=server EXCHANGE=mcx PORT=3002 cargo run"
echo ""

echo "📦 BATCH MODE:"
echo "3. Run NSE Batch Analysis:"
echo "MODE=batch EXCHANGE=nse cargo run"
echo ""

echo "4. Run MCX Batch Analysis:"
echo "MODE=batch EXCHANGE=mcx cargo run"
echo ""

echo "5. Run Both Exchanges Batch:"
echo "MODE=batch EXCHANGE=both cargo run"
echo ""

echo "🐳 DOCKER ORCHESTRATION:"
echo "# Start both services with Docker Compose"
echo "docker-compose up -d"
echo "docker-compose down"
echo "docker-compose logs nse-server"
echo "docker-compose logs mcx-server"
echo ""

echo "⚙️ PROCESS MANAGEMENT:"
echo "# Using PM2"
echo "pm2 start ecosystem.config.js"
echo "pm2 stop ecosystem.config.js"
echo "pm2 monit"
echo ""

echo "# Using systemd (Linux)"
echo "sudo systemctl start nse-analyzer-nse"
echo "sudo systemctl start nse-analyzer-mcx"
echo "sudo systemctl status nse-analyzer-nse"
echo ""

echo "🧪 API TESTING:"
echo ""
echo "# Test NSE APIs (port 3001)"
echo "curl http://localhost:3001/health"
echo "curl http://localhost:3001/api/securities"
echo "curl 'http://localhost:3001/api/contract-info?symbol=NIFTY'"
echo "curl -X POST http://localhost:3001/api/batch-analysis"
echo ""

echo "# Test MCX APIs (port 3002)"
echo "curl http://localhost:3002/health"
echo "curl http://localhost:3002/api/mcx/tickers"
echo "curl 'http://localhost:3002/api/mcx/option-chain?commodity=COPPER&expiry=23DEC2025'"
echo "curl -X POST http://localhost:3002/api/mcx/batch-analysis"
echo ""

echo "🌐 REVERSE PROXY (Nginx):"
echo "# Configure nginx.conf and access via single port"
echo "curl http://localhost/api/securities        # → NSE server"
echo "curl http://localhost/api/mcx/tickers       # → MCX server"
echo ""

echo "🔄 GitHub Actions (CI):"
echo "EXCHANGE=nse cargo run                      # Auto-switches to batch"
echo "EXCHANGE=mcx cargo run                      # Auto-switches to batch"
echo "EXCHANGE=both cargo run                     # Both in batch mode"