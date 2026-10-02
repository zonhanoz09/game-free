# Production Dockerfile for 3v3 Tactical Arena Online PvP
FROM node:20-alpine

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --omit=dev

# Copy server and built WebAssembly game bundle
COPY server.js ./
COPY wasm_dist/ ./wasm_dist/
COPY assets/ ./assets/

# Expose HTTP and WebSocket port
ENV PORT=8080
EXPOSE 8080

# Run the dedicated web & matchmaking server
CMD ["node", "server.js"]
