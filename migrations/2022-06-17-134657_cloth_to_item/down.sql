ALTER TABLE items
    RENAME TO clothes;
ALTER TABLE clothes
    RENAME COLUMN item_name TO cloth_name;
ALTER TABLE uses
    RENAME TO wears;
ALTER TABLE wears
    RENAME COLUMN item_id TO cloth_id;