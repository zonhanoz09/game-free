const http = require('http');
const fs = require('fs');
const path = require('path');
const { WebSocketServer } = require('ws');

const PORT = process.env.PORT || 8080;
const PUBLIC_DIR = path.join(__dirname, 'wasm_dist');

const MIME_TYPES = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'application/javascript; charset=utf-8',
    '.wasm': 'application/wasm',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.wav': 'audio/wav',
    '.mp3': 'audio/mpeg',
    '.css': 'text/css; charset=utf-8',
    '.d.ts': 'text/plain',
};

// HTTP Static Server
const server = http.createServer((req, res) => {
    let reqPath = req.url.split('?')[0];
    if (reqPath === '/' || reqPath === '') {
        reqPath = '/index.html';
    }

    let filePath = path.join(PUBLIC_DIR, reqPath);

    // If requested path does not exist in wasm_dist, check in assets
    if (!fs.existsSync(filePath)) {
        const assetPath = path.join(__dirname, reqPath);
        if (fs.existsSync(assetPath) && fs.statSync(assetPath).isFile()) {
            filePath = assetPath;
        }
    }

    fs.stat(filePath, (err, stats) => {
        if (err || !stats.isFile()) {
            res.writeHead(404, { 'Content-Type': 'text/plain' });
            res.end(`404 Not Found: ${reqPath}`);
            return;
        }

        const ext = path.extname(filePath).toLowerCase();
        const contentType = MIME_TYPES[ext] || 'application/octet-stream';

        res.writeHead(200, {
            'Content-Type': contentType,
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
            'Cache-Control': ext === '.wasm' ? 'public, max-age=3600' : 'no-cache',
        });

        const stream = fs.createReadStream(filePath);
        stream.pipe(res);
    });
});

// WebSocket PvP Room Management
const wss = new WebSocketServer({ server });
const rooms = new Map(); // room_code -> RoomState
let quickMatchQueue = null;

function generateRoomCode() {
    const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
    let code = '';
    for (let i = 0; i < 4; i++) {
        code += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return code;
}

function normalizeType(t) {
    if (!t) return '';
    return t.replace(/([a-z])([A-Z])/g, '$1_$2').toUpperCase();
}

function broadcastToRoom(room, msgObj) {
    const str = JSON.stringify(msgObj);
    if (room.host && room.host.ws.readyState === 1) {
        room.host.ws.send(str);
    }
    if (room.guest && room.guest.ws.readyState === 1) {
        room.guest.ws.send(str);
    }
}

wss.on('connection', (ws) => {
    let currentRoomCode = null;
    let currentRole = null;

    ws.on('message', (raw) => {
        try {
            const data = JSON.parse(raw.toString());
            const rawType = data.type || data.action;
            const type = normalizeType(rawType);
            const payload = data.data || data.payload || data;

            switch (type) {
                case 'CREATE_ROOM': {
                    let code = generateRoomCode();
                    while (rooms.has(code)) {
                        code = generateRoomCode();
                    }
                    currentRoomCode = code;
                    currentRole = 'host';

                    const room = {
                        code,
                        host: { ws, name: payload.name || 'Player 1 (Blue)', hp: 100, lineup: [], ready: false },
                        guest: null,
                        round: 1,
                        results: new Map(),
                    };
                    rooms.set(code, room);

                    ws.send(JSON.stringify({
                        type: 'ROOM_CREATED',
                        room_code: code,
                        role: 'host',
                        player_name: room.host.name,
                    }));
                    console.log(`[PVP SERVER] Room created: ${code} by ${room.host.name}`);
                    break;
                }

                case 'JOIN_ROOM': {
                    const code = (payload.room_code || '').toUpperCase().trim();
                    const room = rooms.get(code);
                    if (!room) {
                        ws.send(JSON.stringify({ type: 'ERROR', message: `Phòng "${code}" không tồn tại!` }));
                        return;
                    }
                    if (room.guest) {
                        ws.send(JSON.stringify({ type: 'ERROR', message: `Phòng "${code}" đã đủ 2 người chơi!` }));
                        return;
                    }

                    currentRoomCode = code;
                    currentRole = 'guest';
                    room.guest = {
                        ws,
                        name: payload.name || 'Player 2 (Red)',
                        hp: 100,
                        lineup: [],
                        ready: false,
                    };

                    // Notify Guest
                    ws.send(JSON.stringify({
                        type: 'ROOM_JOINED',
                        room_code: code,
                        role: 'guest',
                        player_name: room.guest.name,
                        opponent_name: room.host.name,
                        round: room.round,
                    }));

                    // Notify Host
                    room.host.ws.send(JSON.stringify({
                        type: 'OPPONENT_JOINED',
                        room_code: code,
                        role: 'host',
                        player_name: room.host.name,
                        opponent_name: room.guest.name,
                        round: room.round,
                    }));

                    console.log(`[PVP SERVER] ${room.guest.name} joined room ${code} vs ${room.host.name}`);
                    break;
                }

                case 'QUICK_MATCH': {
                    if (quickMatchQueue && quickMatchQueue.readyState === 1 && quickMatchQueue !== ws) {
                        const hostWs = quickMatchQueue;
                        quickMatchQueue = null;
                        const code = generateRoomCode();
                        const room = {
                            code,
                            host: { ws: hostWs, name: 'Player 1 (Blue)', hp: 100, lineup: [], ready: false },
                            guest: { ws, name: 'Player 2 (Red)', hp: 100, lineup: [], ready: false },
                            round: 1,
                            results: new Map(),
                        };
                        rooms.set(code, room);
                        currentRoomCode = code;
                        currentRole = 'guest';

                        hostWs.send(JSON.stringify({
                            type: 'ROOM_JOINED',
                            room_code: code,
                            role: 'host',
                            player_name: room.host.name,
                            opponent_name: room.guest.name,
                            round: 1,
                        }));
                        ws.send(JSON.stringify({
                            type: 'ROOM_JOINED',
                            room_code: code,
                            role: 'guest',
                            player_name: room.guest.name,
                            opponent_name: room.host.name,
                            round: 1,
                        }));
                        console.log(`[PVP SERVER] Quick match formed! Room ${code}`);
                    } else {
                        quickMatchQueue = ws;
                        ws.send(JSON.stringify({ type: 'WAITING_FOR_MATCH', message: 'Đang tìm kiếm đối thủ...' }));
                    }
                    break;
                }

                case 'PLAYER_READY': {
                    const room = rooms.get(currentRoomCode);
                    if (!room) return;

                    const lineup = payload.lineup || [];
                    if (currentRole === 'host') {
                        room.host.lineup = lineup;
                        room.host.ready = true;
                        if (room.guest) {
                            room.guest.ws.send(JSON.stringify({ type: 'OPPONENT_READY' }));
                        }
                    } else if (currentRole === 'guest') {
                        room.guest.lineup = lineup;
                        room.guest.ready = true;
                        if (room.host) {
                            room.host.ws.send(JSON.stringify({ type: 'OPPONENT_READY' }));
                        }
                    }

                    console.log(`[PVP SERVER] Room ${room.code} ${currentRole} is READY with ${lineup.length} units.`);

                    // If both players are ready, start the battle phase!
                    if (room.host.ready && room.guest && room.guest.ready) {
                        room.host.ready = false;
                        room.guest.ready = false;
                        room.results.clear();

                        // Send opponent's lineup to each player
                        room.host.ws.send(JSON.stringify({
                            type: 'START_ROUND',
                            round: room.round,
                            opponent_lineup: room.guest.lineup,
                            player_hp: room.host.hp,
                            opponent_hp: room.guest.hp,
                        }));

                        room.guest.ws.send(JSON.stringify({
                            type: 'START_ROUND',
                            round: room.round,
                            opponent_lineup: room.host.lineup,
                            player_hp: room.guest.hp,
                            opponent_hp: room.host.hp,
                        }));

                        console.log(`[PVP SERVER] Battle started for round ${room.round} in room ${room.code}!`);
                    }
                    break;
                }

                case 'BATTLE_FINISHED': {
                    const room = rooms.get(currentRoomCode);
                    if (!room) return;

                    const winnerRole = payload.winner_role;
                    const survivors = payload.player_survivors || 1;

                    room.results.set(currentRole, { winnerRole, survivors });

                    // When both report (or if host reports), calculate damage
                    if (room.results.size >= 1) {
                        const damage = 10 + survivors * 3;
                        if (winnerRole === 'host') {
                            room.guest.hp = Math.max(0, room.guest.hp - damage);
                        } else if (winnerRole === 'guest') {
                            room.host.hp = Math.max(0, room.host.hp - damage);
                        }

                        // Broadcast HP update
                        broadcastToRoom(room, {
                            type: 'UPDATE_MATCH_HP',
                            host_hp: room.host.hp,
                            guest_hp: room.guest.hp,
                            damage_dealt: damage,
                        });

                        // Check match end
                        if (room.host.hp <= 0 || room.guest.hp <= 0) {
                            const winner = room.host.hp > 0 ? room.host.name : room.guest.name;
                            broadcastToRoom(room, {
                                type: 'MATCH_END',
                                winner,
                            });
                            rooms.delete(room.code);
                        } else {
                            room.round += 1;
                        }
                    }
                    break;
                }
            }
        } catch (err) {
            console.error('[PVP SERVER ERROR] Failed to process message:', err);
        }
    });

    ws.on('close', () => {
        if (quickMatchQueue === ws) {
            quickMatchQueue = null;
        }
        if (currentRoomCode) {
            const room = rooms.get(currentRoomCode);
            if (room) {
                if (currentRole === 'host') {
                    if (room.guest) {
                        room.guest.ws.send(JSON.stringify({ type: 'ERROR', message: 'Chủ phòng đã thoát trận.' }));
                    }
                    rooms.delete(currentRoomCode);
                } else if (currentRole === 'guest') {
                    if (room.host) {
                        room.host.ws.send(JSON.stringify({ type: 'ERROR', message: 'Đối thủ đã thoát phòng.' }));
                    }
                    room.guest = null;
                }
            }
        }
    });
});

server.listen(PORT, '0.0.0.0', () => {
    console.log('=======================================================');
    console.log(' ⚔️ 3v3 TACTICAL ARENA - ONLINE PVP SERVER RUNNING');
    console.log(` 🌐 Web Client & WebSocket URL: http://localhost:${PORT}`);
    console.log(` 📦 Serving assets from: ${PUBLIC_DIR}`);
    console.log('=======================================================');
});
