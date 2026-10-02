const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const DATA_DIR = path.join(__dirname, 'data');
const DB_FILE = path.join(DATA_DIR, 'game_db.json');

class GameDatabase {
    constructor() {
        this.data = {
            users: {},
            matches: []
        };
        this.init();
    }

    init() {
        if (!fs.existsSync(DATA_DIR)) {
            fs.mkdirSync(DATA_DIR, { recursive: true });
        }

        if (fs.existsSync(DB_FILE)) {
            try {
                const raw = fs.readFileSync(DB_FILE, 'utf-8');
                this.data = JSON.parse(raw);
                if (!this.data.users) this.data.users = {};
                if (!this.data.matches) this.data.matches = [];
                console.log(`[OCI DATABASE] Loaded ${Object.keys(this.data.users).length} users and ${this.data.matches.length} matches from disk.`);
            } catch (err) {
                console.error('[OCI DATABASE] Error parsing existing db, resetting backup:', err);
                this.saveSync();
            }
        } else {
            console.log('[OCI DATABASE] Initializing brand new database file...');
            this.saveSync();
        }
    }

    saveSync() {
        const tmp = DB_FILE + '.tmp';
        fs.writeFileSync(tmp, JSON.stringify(this.data, null, 2), 'utf-8');
        fs.renameSync(tmp, DB_FILE);
    }

    async save() {
        const tmp = DB_FILE + '.tmp';
        await fs.promises.writeFile(tmp, JSON.stringify(this.data, null, 2), 'utf-8');
        await fs.promises.rename(tmp, DB_FILE);
    }

    hashPassword(pwd) {
        return crypto.createHash('sha256').update(pwd + '_tac_arena_salt').digest('hex');
    }

    sanitizeUser(user) {
        if (!user) return null;
        const { passwordHash, ...safeUser } = user;
        return safeUser;
    }

    async register(username, password, displayName, avatar) {
        const cleanUser = (username || '').trim().toLowerCase();
        if (!cleanUser || cleanUser.length < 3 || cleanUser.length > 20) {
            throw new Error('Tên tài khoản phải từ 3 đến 20 ký tự!');
        }
        if (!/^[a-zA-Z0-9_]+$/.test(cleanUser)) {
            throw new Error('Tên tài khoản chỉ được chứa chữ cái, số và dấu gạch dưới!');
        }
        if (!password || password.length < 4) {
            throw new Error('Mật khẩu phải từ 4 ký tự trở lên!');
        }

        if (this.data.users[cleanUser]) {
            throw new Error('Tên tài khoản này đã được sử dụng!');
        }

        const newUser = {
            username: cleanUser,
            displayName: (displayName || cleanUser).trim().slice(0, 24),
            avatar: avatar || 'knight',
            passwordHash: this.hashPassword(password),
            elo: 1000,
            wins: 0,
            losses: 0,
            matches: 0,
            createdAt: new Date().toISOString(),
            lastLogin: new Date().toISOString()
        };

        this.data.users[cleanUser] = newUser;
        await this.save();
        console.log(`[OCI DATABASE] Registered new user: ${cleanUser} (${newUser.displayName})`);
        return this.sanitizeUser(newUser);
    }

    async login(username, password) {
        const cleanUser = (username || '').trim().toLowerCase();
        const user = this.data.users[cleanUser];
        if (!user) {
            throw new Error('Tài khoản không tồn tại!');
        }

        const hash = this.hashPassword(password);
        if (user.passwordHash !== hash) {
            throw new Error('Mật khẩu không chính xác!');
        }

        user.lastLogin = new Date().toISOString();
        await this.save();
        console.log(`[OCI DATABASE] User logged in: ${cleanUser}`);
        return this.sanitizeUser(user);
    }

    getUser(username) {
        const cleanUser = (username || '').trim().toLowerCase();
        return this.sanitizeUser(this.data.users[cleanUser]);
    }

    async recordMatch(matchId, hostUser, guestUser, winnerUser, rounds) {
        const host = this.data.users[(hostUser || '').toLowerCase()];
        const guest = this.data.users[(guestUser || '').toLowerCase()];

        const record = {
            matchId: matchId || 'm_' + Date.now(),
            host: hostUser,
            guest: guestUser,
            winner: winnerUser,
            rounds: rounds || 1,
            timestamp: new Date().toISOString()
        };

        let eloChange = 25;
        if (winnerUser) {
            const isHostWinner = winnerUser.toLowerCase() === (hostUser || '').toLowerCase();
            if (host) {
                host.matches += 1;
                if (isHostWinner) {
                    host.wins += 1;
                    host.elo += eloChange;
                } else {
                    host.losses += 1;
                    host.elo = Math.max(500, host.elo - 20);
                }
            }

            if (guest) {
                guest.matches += 1;
                if (!isHostWinner) {
                    guest.wins += 1;
                    guest.elo += eloChange;
                } else {
                    guest.losses += 1;
                    guest.elo = Math.max(500, guest.elo - 20);
                }
            }
        }

        this.data.matches.unshift(record);
        if (this.data.matches.length > 200) {
            this.data.matches = this.data.matches.slice(0, 200);
        }

        await this.save();
        console.log(`[OCI DATABASE] Recorded match ${record.matchId}: Winner=${winnerUser}`);
        return {
            record,
            host: this.sanitizeUser(host),
            guest: this.sanitizeUser(guest)
        };
    }

    getLeaderboard(limit = 20) {
        const list = Object.values(this.data.users).map(u => this.sanitizeUser(u));
        list.sort((a, b) => b.elo - a.elo || b.wins - a.wins);
        return list.slice(0, limit);
    }
}

module.exports = new GameDatabase();
