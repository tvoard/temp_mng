-- 사용자 종류 (역할) 테이블
CREATE TABLE IF NOT EXISTS user_type
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT                                NOT NULL UNIQUE, -- 예: SuperAdmin, ContentEditor, Viewer
    description TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 업데이트 시 updated_at 자동 갱신 트리거 (SQLite)
CREATE TRIGGER IF NOT EXISTS user_type_updated_at
    AFTER UPDATE
    ON user_type
    FOR EACH ROW
BEGIN
    UPDATE user_type SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- 관리자 사용자 테이블
CREATE TABLE IF NOT EXISTS admin_user
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT                                NOT NULL UNIQUE,
    password_hash TEXT                                NOT NULL,
    user_type_id  INTEGER                             NOT NULL REFERENCES user_type (id) ON DELETE RESTRICT, -- 사용자 종류 삭제 시 해당 유저가 있으면 삭제 방지
    is_active     BOOLEAN   DEFAULT TRUE              NOT NULL,
    last_login_at TIMESTAMP,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_admin_user_user_type_id ON admin_user (user_type_id);

CREATE TRIGGER IF NOT EXISTS admin_user_updated_at
    AFTER UPDATE
    ON admin_user
    FOR EACH ROW
BEGIN
    UPDATE admin_user SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- 권한 테이블
CREATE TABLE IF NOT EXISTS permission
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT                                NOT NULL UNIQUE, -- 예: user:create, content:edit, report:view
    description TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TRIGGER IF NOT EXISTS permission_updated_at
    AFTER UPDATE
    ON permission
    FOR EACH ROW
BEGIN
    UPDATE permission SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- 메뉴 항목 테이블
CREATE TABLE IF NOT EXISTS menu_item
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT                                NOT NULL,                                     -- 메뉴 표시 이름 (예: "사용자 관리")
    path          TEXT                                NOT NULL UNIQUE,                              -- 프론트엔드 라우팅 경로 (예: "/users")
    icon          TEXT,                                                                             -- 아이콘 클래스 또는 URL (선택 사항)
    parent_id     INTEGER                             REFERENCES menu_item (id) ON DELETE SET NULL, -- 부모 메뉴 ID (null이면 최상위)
    display_order INTEGER   DEFAULT 0                 NOT NULL,                                     -- 표시 순서
    is_visible    BOOLEAN   DEFAULT TRUE              NOT NULL,                                     -- 메뉴 표시 여부
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_menu_item_parent_id ON menu_item (parent_id);

CREATE TRIGGER IF NOT EXISTS menu_item_updated_at
    AFTER UPDATE
    ON menu_item
    FOR EACH ROW
BEGIN
    UPDATE menu_item SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- 사용자 종류 - 권한 매핑 테이블 (다대다)
CREATE TABLE IF NOT EXISTS user_type_permission
(
    user_type_id  INTEGER NOT NULL REFERENCES user_type (id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permission (id) ON DELETE CASCADE,
    PRIMARY KEY (user_type_id, permission_id)
);

-- 사용자 종류 - 메뉴 매핑 테이블 (다대다)
CREATE TABLE IF NOT EXISTS user_type_menu
(
    user_type_id INTEGER NOT NULL REFERENCES user_type (id) ON DELETE CASCADE,
    menu_item_id INTEGER NOT NULL REFERENCES menu_item (id) ON DELETE CASCADE,
    PRIMARY KEY (user_type_id, menu_item_id)
);

-- 초기 데이터 (선택 사항)
INSERT INTO user_type (name, description)
VALUES ('SuperAdmin', 'System super administrator with all permissions');
INSERT INTO permission (code, description)
VALUES ('*', 'All permissions');
INSERT INTO menu_item (name, path, display_order)
VALUES ('Dashboard', '/', 0);

-- SuperAdmin에게 모든 권한과 기본 메뉴 부여
INSERT INTO user_type_permission (user_type_id, permission_id)
VALUES (1, 1);
INSERT INTO user_type_menu (user_type_id, menu_item_id)
VALUES (1, 1);

-- 예시 관리자 (bcrypt 해싱된 비밀번호 사용 - 실제로는 API 통해 생성)
-- 'password123' 해시 예시 (실제로는 프로그램에서 생성된 해시 사용)
-- INSERT INTO admin_user (username, password_hash, user_type_id) VALUES ('admin', '$2b$12$your_bcrypt_hash_here', 1);