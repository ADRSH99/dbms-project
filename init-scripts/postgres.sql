CREATE TABLE orders (
    id INT PRIMARY KEY,
    user_id INT,
    amount FLOAT,
    product VARCHAR(255)
);
CREATE TABLE customers (
    id INT PRIMARY KEY,
    name VARCHAR(255),
    email VARCHAR(255)
);
CREATE TABLE reviews (
    id INT PRIMARY KEY,
    order_id INT,
    rating INT,
    comment TEXT
);
INSERT INTO customers (id, name, email)
VALUES (1, 'Alice', 'alice@example.com'),
    (2, 'Bob', 'bob@example.com');
INSERT INTO orders (id, user_id, amount, product)
VALUES (1, 1, 1500.50, 'Laptop'),
    (2, 2, 45.00, 'Mouse'),
    (3, 1, 120.00, 'Keyboard');
INSERT INTO reviews (id, order_id, rating, comment)
VALUES (1, 1, 5, 'Great laptop!'),
    (2, 2, 4, 'Good mouse');