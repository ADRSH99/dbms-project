db = db.getSiblingDB('iceweave');

db.inventory.insertMany([
    { item: "Laptop", qty: 50 },
    { item: "Mouse", qty: 200 },
    { item: "Keyboard", qty: 150 }
]);

db.stock_logs.insertMany([
    { item: "Laptop", action: "restock", amount: 50, date: "2026-03-01" },
    { item: "Mouse", action: "restock", amount: 200, date: "2026-03-02" }
]);
