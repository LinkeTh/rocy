-- Add migration script here

CREATE TABLE IF NOT EXISTS user_receipts
(
    id       SERIAL PRIMARY KEY,
    image_id INT NOT NULL REFERENCES user_images (id) ON DELETE CASCADE,
    json     TEXT
);

CREATE INDEX IF NOT EXISTS idx_user_receipts_image_id ON user_receipts (image_id);
