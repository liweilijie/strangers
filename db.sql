CREATE TABLE "medicinal" (
                             "id" SERIAL PRIMARY KEY,
                             "category" varchar NOT NULL,
                             "name" varchar NOT NULL,
                             "batch_number" varchar,
                             "count" int NOT NULL,
                             "validity" date NOT NULL,
                             is_del BOOLEAN NOT NULL DEFAULT FALSE,
                             "created_at" date DEFAULT (now())
);

CREATE INDEX ON "medicinal" ("name");

CREATE INDEX ON "medicinal" ("batch_number");

CREATE UNIQUE INDEX ON "medicinal" ("id");

// 辅助
select * from medicinal;
insert into medicinal(category, name, batch_number, count, validity) values('测试手术类目', '多巴胺', '2008094', 3, '2022-07-01');