import re

def update_login_ui():
    with open('dist/wasm/index.html', 'r', encoding='utf-8') as f:
        content = f.read()

    auth_css = '''
        /* ========================================== */
        /* AUTH SCREEN & LOGIN MODAL (PREMIUM GAMING UI) */
        /* ========================================== */
        #auth-modal {
            background: radial-gradient(circle at center, rgba(15, 23, 42, 0.88) 0%, rgba(3, 7, 18, 0.96) 100%);
            backdrop-filter: blur(14px);
        }

        #auth-modal .modal-card {
            max-width: 480px;
            background: linear-gradient(180deg, #0f172a 0%, #090d16 100%);
            border: 1px solid rgba(56, 189, 248, 0.35);
            border-radius: 16px;
            box-shadow: 0 0 50px rgba(2, 132, 199, 0.25), 0 25px 50px -12px rgba(0, 0, 0, 0.95);
            position: relative;
            overflow: hidden;
        }

        #auth-modal .modal-card::before {
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 300%;
            height: 3px;
            background: linear-gradient(90deg, transparent, #38bdf8, #facc15, #818cf8, transparent);
            animation: authTopBorderShine 4s linear infinite;
        }

        @keyframes authTopBorderShine {
            0% { transform: translateX(0); }
            100% { transform: translateX(33.33%); }
        }

        .auth-banner-header {
            padding: 24px 24px 16px;
            text-align: center;
            background: radial-gradient(ellipse at top, rgba(30, 58, 138, 0.35) 0%, transparent 70%);
            border-bottom: 1px solid rgba(51, 65, 85, 0.5);
            position: relative;
        }

        .auth-crest {
            font-size: 2.5rem;
            display: inline-block;
            filter: drop-shadow(0 0 16px rgba(56, 189, 248, 0.65));
            margin-bottom: 6px;
            animation: crestFloat 3s ease-in-out infinite;
        }

        @keyframes crestFloat {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-4px); }
        }

        .auth-main-title {
            font-size: 1.4rem;
            font-weight: 900;
            letter-spacing: 1.5px;
            text-transform: uppercase;
            background: linear-gradient(135deg, #ffffff 0%, #38bdf8 50%, #facc15 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 4px;
        }

        .auth-subtitle {
            font-size: 0.8rem;
            color: #94a3b8;
            letter-spacing: 0.5px;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
        }

        .auth-cloud-badge {
            background: rgba(30, 41, 59, 0.9);
            border: 1px solid rgba(56, 189, 248, 0.4);
            color: #38bdf8;
            font-size: 0.7rem;
            font-weight: 700;
            padding: 2px 7px;
            border-radius: 12px;
            text-transform: uppercase;
            letter-spacing: 0.8px;
        }

        .auth-tabs-wrap {
            display: flex;
            background: #090d16;
            padding: 4px;
            border-radius: 12px;
            border: 1px solid #1e293b;
            gap: 6px;
            margin-bottom: 10px;
        }

        .auth-tab-btn {
            flex: 1;
            padding: 10px 14px;
            font-size: 0.9rem;
            font-weight: 700;
            color: #94a3b8;
            background: transparent;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.25s ease;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
        }

        .auth-tab-btn:hover {
            color: #f8fafc;
            background: rgba(30, 41, 59, 0.5);
        }

        .auth-tab-btn.active {
            color: #ffffff;
            background: linear-gradient(135deg, #0284c7 0%, #2563eb 100%);
            box-shadow: 0 4px 14px rgba(2, 132, 199, 0.4);
        }

        .auth-input-group {
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        .auth-input-label {
            font-size: 0.82rem;
            font-weight: 600;
            color: #cbd5e1;
            display: flex;
            align-items: center;
            gap: 6px;
        }

        .auth-input-field {
            background: rgba(15, 23, 42, 0.85);
            border: 1px solid #334155;
            color: #ffffff;
            font-size: 0.95rem;
            padding: 11px 14px;
            border-radius: 8px;
            transition: all 0.2s ease;
            outline: none;
        }

        .auth-input-field:focus {
            border-color: #38bdf8;
            background: rgba(24, 34, 53, 0.95);
            box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.25), 0 0 12px rgba(56, 189, 248, 0.15);
        }

        .auth-input-field::placeholder {
            color: #475569;
        }

        /* Avatar Picker Grid */
        .auth-avatar-grid {
            display: grid;
            grid-template-columns: repeat(5, 1fr);
            gap: 8px;
        }

        .auth-avatar-card {
            background: rgba(19, 28, 49, 0.8);
            border: 2px solid #1e293b;
            border-radius: 10px;
            padding: 8px 4px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
            position: relative;
        }

        .auth-avatar-card:hover {
            border-color: #38bdf8;
            background: rgba(30, 41, 59, 0.9);
            transform: translateY(-2px);
        }

        .auth-avatar-card.selected {
            border-color: #facc15;
            background: linear-gradient(180deg, rgba(30, 58, 138, 0.4) 0%, rgba(202, 138, 4, 0.15) 100%);
            box-shadow: 0 0 14px rgba(250, 204, 21, 0.35);
            transform: scale(1.05);
        }

        .auth-avatar-icon {
            font-size: 1.6rem;
            margin-bottom: 2px;
        }

        .auth-avatar-label {
            font-size: 0.68rem;
            font-weight: 700;
            color: #94a3b8;
            text-align: center;
            text-transform: uppercase;
        }

        .auth-avatar-card.selected .auth-avatar-label {
            color: #facc15;
        }

        .auth-submit-btn {
            width: 100%;
            padding: 12px 18px;
            font-size: 0.98rem;
            font-weight: 800;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            color: #ffffff;
            background: linear-gradient(135deg, #0284c7 0%, #0284c7 35%, #2563eb 70%, #4f46e5 100%);
            border: 1px solid #38bdf8;
            border-radius: 10px;
            cursor: pointer;
            box-shadow: 0 4px 16px rgba(2, 132, 199, 0.45);
            transition: all 0.25s ease;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
        }

        .auth-submit-btn:hover {
            transform: translateY(-1px);
            box-shadow: 0 6px 22px rgba(2, 132, 199, 0.6);
            background: linear-gradient(135deg, #0369a1 0%, #0284c7 35%, #1d4ed8 70%, #4338ca 100%);
        }

        .auth-submit-btn:active {
            transform: translateY(1px);
        }

        .auth-guest-btn {
            background: rgba(15, 23, 42, 0.6);
            border: 1px solid #334155;
            color: #94a3b8;
            padding: 10px 16px;
            border-radius: 8px;
            font-size: 0.85rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            text-align: center;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
        }

        .auth-guest-btn:hover {
            border-color: #64748b;
            color: #e2e8f0;
            background: rgba(30, 41, 59, 0.8);
        }

        .auth-info-banner {
            background: rgba(15, 23, 42, 0.75);
            border: 1px solid rgba(56, 189, 248, 0.25);
            border-radius: 8px;
            padding: 10px 12px;
            display: flex;
            align-items: center;
            gap: 10px;
            font-size: 0.78rem;
            color: #94a3b8;
            line-height: 1.4;
        }

        .auth-info-banner-icon {
            font-size: 1.25rem;
            flex-shrink: 0;
        }

        .auth-error-box {
            background: rgba(239, 68, 68, 0.15);
            border: 1px solid rgba(239, 68, 68, 0.45);
            color: #fca5a5;
            padding: 9px 12px;
            border-radius: 8px;
            font-size: 0.82rem;
            font-weight: 600;
            display: none;
            align-items: center;
            gap: 6px;
            animation: errorShake 0.3s ease;
        }

        @keyframes errorShake {
            0%, 100% { transform: translateX(0); }
            20%, 60% { transform: translateX(-5px); }
            40%, 80% { transform: translateX(5px); }
        }
    '''

    # Insert CSS before </style>
    if 'AUTH SCREEN & LOGIN MODAL' not in content:
        content = content.replace('</style>', auth_css + '\n    </style>')

    # Replace modal HTML
    old_start = '    <!-- Modal: MÀN HÌNH ĐĂNG NHẬP / ĐĂNG KÝ (OCI DATABASE) -->'
    old_end = '    <!-- Modal: SẢNH ĐỐI KHÁNG 2 NGƯỜI (PvP MATCHMAKING HUB) -->'

    idx1 = content.find(old_start)
    idx2 = content.find(old_end)
    assert idx1 != -1 and idx2 != -1, f'Indices not found: {idx1}, {idx2}'

    new_auth_modal = '''    <!-- Modal: MÀN HÌNH ĐĂNG NHẬP / ĐĂNG KÝ (OCI DATABASE) -->
    <div id="auth-modal" class="modal-overlay">
        <div class="modal-card">
            <div class="auth-banner-header">
                <button class="modal-close-btn" id="btn-close-auth" style="position:absolute; top:12px; right:14px;" title="Đóng (Esc)">✕</button>
                <div class="auth-crest">⚔️</div>
                <h1 class="auth-main-title">Tactical Arena</h1>
                <div class="auth-subtitle">
                    <span>Đấu Trường Chiến Thuật Đối Kháng</span>
                    <span class="auth-cloud-badge">OCI CLOUD</span>
                </div>
            </div>

            <div class="modal-body" style="padding: 18px 24px 24px; gap: 14px;">
                <!-- Tab Selector -->
                <div class="auth-tabs-wrap">
                    <button id="tab-login" class="auth-tab-btn active">
                        <span>🔑</span>
                        <span>Đăng Nhập</span>
                    </button>
                    <button id="tab-register" class="auth-tab-btn">
                        <span>📝</span>
                        <span>Đăng Ký Mới</span>
                    </button>
                </div>

                <!-- Username -->
                <div class="auth-input-group">
                    <label class="auth-input-label">
                        <span>👤</span>
                        <span>Tên tài khoản (Username)</span>
                    </label>
                    <input type="text" id="auth-username" class="auth-input-field" placeholder="Nhập tên tài khoản..." maxlength="20" autocomplete="username">
                </div>

                <!-- Password -->
                <div class="auth-input-group">
                    <label class="auth-input-label">
                        <span>🔒</span>
                        <span>Mật khẩu bảo mật</span>
                    </label>
                    <input type="password" id="auth-password" class="auth-input-field" placeholder="Nhập mật khẩu (tối thiểu 4 ký tự)..." autocomplete="current-password">
                </div>

                <!-- Extra Fields for Registration -->
                <div id="register-fields" style="display: none; flex-direction: column; gap: 14px;">
                    <div class="auth-input-group">
                        <label class="auth-input-label">
                            <span>🏷️</span>
                            <span>Tên hiển thị trong game (Biệt danh)</span>
                        </label>
                        <input type="text" id="auth-display-name" class="auth-input-field" placeholder="Ví dụ: Chiến Thần Sấm..." maxlength="24">
                    </div>

                    <div class="auth-input-group">
                        <label class="auth-input-label">
                            <span>👑</span>
                            <span>Chọn Tướng đại diện (Avatar)</span>
                        </label>
                        <div class="auth-avatar-grid" id="avatar-selector">
                            <div class="auth-avatar-card avatar-option selected" data-avatar="knight" title="Hiệp Sĩ (Knight)">
                                <span class="auth-avatar-icon">🛡️</span>
                                <span class="auth-avatar-label">Hiệp Sĩ</span>
                            </div>
                            <div class="auth-avatar-card avatar-option" data-avatar="archer" title="Cung Thủ (Archer)">
                                <span class="auth-avatar-icon">🏹</span>
                                <span class="auth-avatar-label">Cung Thủ</span>
                            </div>
                            <div class="auth-avatar-card avatar-option" data-avatar="mage" title="Pháp Sư (Mage)">
                                <span class="auth-avatar-icon">🔮</span>
                                <span class="auth-avatar-label">Pháp Sư</span>
                            </div>
                            <div class="auth-avatar-card avatar-option" data-avatar="assassin" title="Sát Thủ (Assassin)">
                                <span class="auth-avatar-icon">🗡️</span>
                                <span class="auth-avatar-label">Sát Thủ</span>
                            </div>
                            <div class="auth-avatar-card avatar-option" data-avatar="cleric" title="Mục Sư (Cleric)">
                                <span class="auth-avatar-icon">✨</span>
                                <span class="auth-avatar-label">Mục Sư</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Info Cloud Banner -->
                <div class="auth-info-banner">
                    <span class="auth-info-banner-icon">☁️</span>
                    <span>Tài khoản, điểm xếp hạng ELO và lịch sử đấu được lưu trữ bảo mật vĩnh viễn trên cơ sở dữ liệu <strong>OCI Free Tier</strong>.</span>
                </div>

                <!-- Error Message -->
                <div id="auth-error-msg" class="auth-error-box"></div>

                <!-- Action Buttons -->
                <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 4px;">
                    <button id="btn-auth-submit" class="auth-submit-btn">
                        <span>⚡</span>
                        <span id="btn-auth-submit-text">ĐĂNG NHẬP NGAY</span>
                    </button>
                    <button id="btn-auth-guest" class="auth-guest-btn">
                        <span>🎮</span>
                        <span>Chơi thử với tư cách Khách</span>
                    </button>
                </div>
            </div>
        </div>
    </div>\n\n'''

    content = content[:idx1] + new_auth_modal + content[idx2:]

    # Update tab click handlers
    old_tab_login = "document.getElementById('tab-login').className = 'btn btn-primary';"
    if old_tab_login in content:
        content = content.replace(
            "document.getElementById('tab-login').className = 'btn btn-primary';",
            "document.getElementById('tab-login').classList.add('active');"
        )
        content = content.replace(
            "document.getElementById('tab-register').className = 'btn';",
            "document.getElementById('tab-register').classList.remove('active');"
        )
        content = content.replace(
            "document.getElementById('tab-register').className = 'btn btn-primary';",
            "document.getElementById('tab-register').classList.add('active');"
        )
        content = content.replace(
            "document.getElementById('tab-login').className = 'btn';",
            "document.getElementById('tab-login').classList.remove('active');"
        )

    # Update submit button text target
    content = content.replace(
        "document.getElementById('btn-auth-submit').textContent = 'Đăng Nhập Ngay';",
        "const t1 = document.getElementById('btn-auth-submit-text'); if(t1) t1.textContent = 'ĐĂNG NHẬP NGAY'; else document.getElementById('btn-auth-submit').textContent = 'Đăng Nhập Ngay';"
    )
    content = content.replace(
        "document.getElementById('btn-auth-submit').textContent = 'Tạo Tài Khoản & Lưu Vào OCI';",
        "const t2 = document.getElementById('btn-auth-submit-text'); if(t2) t2.textContent = 'TẠO TÀI KHOẢN & LƯU VÀO OCI'; else document.getElementById('btn-auth-submit').textContent = 'Tạo Tài Khoản & Lưu Vào OCI';"
    )

    # Update errorDiv display
    content = content.replace(
        "errorDiv.textContent = 'Vui lòng nhập đầy đủ tên tài khoản và mật khẩu!';",
        "errorDiv.innerHTML = '<span>⚠️</span><span>Vui lòng nhập đầy đủ tên tài khoản và mật khẩu!</span>';"
    )
    content = content.replace(
        "errorDiv.style.display = 'block';",
        "errorDiv.style.display = 'flex';"
    )

    with open('dist/wasm/index.html', 'w', encoding='utf-8') as f:
        f.write(content)

    print('Successfully applied high-end login styling to dist/wasm/index.html!')

if __name__ == '__main__':
    update_login_ui()
