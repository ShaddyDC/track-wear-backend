CREATE TABLE item_inventory (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    movement INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    update_time TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_items FOREIGN KEY(item_id) REFERENCES items(id)
);
INSERT INTO item_inventory(movement, item_id)
SELECT 1,
    id
FROM items;