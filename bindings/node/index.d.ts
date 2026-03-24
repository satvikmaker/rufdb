/**
 * @rsdb/node — Node.js bindings for rsdb
 *
 * A fast, file-system based JSON database written in Rust.
 * Drop-in replacement for lowdb with native performance.
 */

/**
 * A synchronous JSON file database.
 *
 * @example
 * ```typescript
 * const db = new RsDb('db.json', { posts: [], users: [] });
 * db.read();
 * console.log(db.getData()); // { posts: [], users: [] }
 *
 * db.setData({ posts: [{ id: 1, title: 'Hello' }], users: [] });
 * db.write();
 * ```
 */
export class RsDb {
  /**
   * Create a new database instance.
   * @param path - Path to the JSON file
   * @param defaultData - Default data used when the file doesn't exist
   */
  constructor(path: string, defaultData: any);

  /** Read data from the JSON file into memory. */
  read(): void;

  /** Write the current in-memory data to the JSON file. */
  write(): void;

  /** Get the current data as a JavaScript object. */
  getData(): any;

  /** Replace the in-memory data with a new object. */
  setData(data: any): void;

  /**
   * Apply a JSON merge patch (RFC 7396) to the data, then write to disk.
   *
   * - Object keys in the patch are merged into existing data
   * - `null` values delete the corresponding key
   * - Non-object patches replace the entire data
   */
  update(patch: any): void;
}

/**
 * Read a JSON database file and return its contents.
 * @param path - Path to the JSON file
 * @param defaultData - Default data if file doesn't exist
 */
export function readDb(path: string, defaultData: any): any;

/**
 * Write data to a JSON database file.
 * @param path - Path to the JSON file
 * @param data - Data to write
 */
export function writeDb(path: string, data: any): void;
