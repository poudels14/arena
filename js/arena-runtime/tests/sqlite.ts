import { Flags, Client } from "@arena/runtime/sqlite";

const client = new Client({
  path: "./db.sqlite",
  flags: Flags.SQLITE_OPEN_CREATE | Flags.SQLITE_OPEN_READ_WRITE,
});

await client.query(`CREATE TABLE IF NOT EXISTS person (
  id    INTEGER PRIMARY KEY,
  name  TEXT NOT NULL,
  data  TEXT
)`);

let rows = await client.query(`SELECT * FROM person;`);
console.log(rows);

await client
  .transaction(async () => {
    await client.query(`INSERT INTO person (name, data) VALUES (?, ?)`, [
      "my name",
      "I am FINALLY lost!",
    ]);
  })
  .catch((e) => {
    console.error(e);
  });

rows = await client.query(`SELECT * FROM person;`);
console.log(rows);
