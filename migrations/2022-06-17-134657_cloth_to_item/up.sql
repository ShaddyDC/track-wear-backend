ALTER TABLE clothes
    RENAME TO items;
ALTER TABLE items
    RENAME COLUMN cloth_name TO item_name;
ALTER TABLE wears
    RENAME TO uses;
ALTER TABLE uses
    RENAME COLUMN cloth_id TO item_id;