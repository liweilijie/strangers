CREATE TABLE "admin" (
                         "id" SERIAL PRIMARY KEY,
                         "username" varchar(50) UNIQUE NOT NULL,
                         "password" varchar(255) NOT NULL,
                         "is_sys" boolean NOT NULL DEFAULT FALSE,
                         "is_del" boolean NOT NULL DEFAULT FALSE
);


CREATE TABLE "medicinal" (
                             "id" SERIAL PRIMARY KEY,
                             "category" varchar NOT NULL,
                             "name" varchar NOT NULL,
                             "batch_number" varchar,
                             "spec" varchar,
                             "count" varchar,
                             "validity" date NOT NULL,
                             is_del BOOLEAN NOT NULL DEFAULT FALSE,
                             "notify_at" TIMESTAMPTZ NOT NULL DEFAULT (now()),
                             "created_at" date DEFAULT (now())
);

CREATE INDEX ON "medicinal" ("name");

CREATE INDEX ON "medicinal" ("validity");

CREATE UNIQUE INDEX ON "medicinal" ("id");


show timezone;
set timezone = 'Asia/Shanghai';
// 辅助
insert into admin(username, password, is_sys, is_del) values('wgr', '$2b$12$QW8Lmf0gvsb1xtRJLxJxzea2M2p5Pxx1LrmPuVzria5obcY8u890C', true, false);

select * from medicinal;
insert into medicinal(category, name, batch_number, count, validity) values('测试手术类目', '多巴胺', '2008094', '3', '2022-03-01');
insert into medicinal(category, name, batch_number, count, validity) values('测试手术类目', 'N95口罩01', '5009094', '300', '2022-04-08');

select * from medicinal where is_del = false and validity <= '2022-03-29' and notify_at <= '2022-03-29 11:37:00';

update medicinal set notify_at = '2022-03-28 10:47:27' where id = 2;

alter table medicinal ADD spec varchar;
alter table medicinal ADD spec varchar default 'Empty' not null;


