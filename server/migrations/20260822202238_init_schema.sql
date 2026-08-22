CREATE TABLE users (
	id            INTEGER PRIMARY KEY AUTOINCREMENT,
	username	  TEXT NOT NULL UNIQUE,
	password_hash TEXT NOT NULL,
	created_at	  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE doorbells (
	id			  TEXT PRIMARY KEY,
	name		  TEXT NOT NULL,
	created_at	  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE push_subscriptions (
	id			  INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id		  INTEGER NOT NULL,
	endpoint	  TEXT NOT NULL UNIQUE,
	p256dh		  TEXT NOT NULL,
	auth		  TEXT NOT NULL,
	created_at	  TEXT NOT NULL DEFAULT (datetime('now')),
	FOREIGN KEY	  (user_id) REFERENCES users(id) ON DELETE CASCADE
);
