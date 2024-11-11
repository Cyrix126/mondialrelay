 CREATE TABLE shipments (
   id SERIAL PRIMARY KEY,
   order_id INT NOT NULL,
   label_url TEXT NOT NULL UNIQUE,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
