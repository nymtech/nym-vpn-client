// Writes the nym-vpn-client code shard to the public retrieval store, which the
// Nym docs MCP route searches alongside the nym shards (search_code). The store,
// its roles and its CI setup are described in the nym repository:
// documentation/MCP-STORE.md and documentation/MCP-STORE-SETUP.md.
//
// Run from .github/indexer:
//   node index.mts --dry-run       walk, chunk and check the gates; write nothing
//   RETRIEVAL_STORE_WRITE_URL=... VOYAGE_API_KEY=... node index.mts
//
// The store skips a chunk whose id and content hash it already holds, so a run
// embeds only the chunks that changed since the last one.

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import pg from 'pg';
import { chunkCodeFile } from '@nymproject/retrieval-indexer/code-chunker';
import { voyageProvider } from '@nymproject/retrieval-indexer/embed';
import { indexShard } from '@nymproject/retrieval-indexer/store';
import { assertNoSilentDrops, assertRootsYieldChunks, collectSourceFiles } from '@nymproject/retrieval-indexer/walk';

const REPO = 'nym-vpn-client';
const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
// wireguard holds the Nym-maintained Go netstack and forwarders (libwg).
// nym-vpn-windows is C++, which the package does not chunk, so it would add no chunks.
const ROOTS = ['nym-vpn-core', 'nym-vpn-app', 'nym-vpn-android', 'nym-vpn-apple', 'wireguard'];
// The model the MCP route embeds code queries with; one model per modality.
const CODE_MODEL = 'voyage-code-3';
// A run that would delete more than this share of the shard is refused, so a
// broken walk cannot empty part of the public index. ALLOW_SHRINK=1 lets an
// intended large removal through.
const MAX_SHRINK = 0.2;

async function main(): Promise<void> {
  const dryRun = process.argv.includes('--dry-run');

  const { files, oversized, dropped } = collectSourceFiles(REPO_ROOT, ROOTS);
  assertNoSilentDrops(dropped);
  const chunks = (
    await Promise.all(
      files.map((f) => chunkCodeFile(f.content, f.path, { repo: REPO, source: `${REPO}-code`, github: `https://github.com/nymtech/${REPO}/blob/develop` })),
    )
  ).flat();
  assertRootsYieldChunks(ROOTS, files, chunks.map((c) => c.title));
  const accepted = Object.entries(dropped).map(([ext, f]) => `.${ext} ${f.length}`).join(', ');
  console.log(`${files.length} source files in ${ROOTS.length} roots -> ${chunks.length} chunks (not chunked, accepted: ${accepted || 'none'}; oversized: ${oversized.length})`);
  if (dryRun) return;

  const url = process.env.RETRIEVAL_STORE_WRITE_URL;
  const apiKey = process.env.VOYAGE_API_KEY;
  if (!url || !apiKey) throw new Error('index: set RETRIEVAL_STORE_WRITE_URL and VOYAGE_API_KEY, or pass --dry-run');

  // One client, not a pool: a write run is one transaction on one connection.
  const db = new pg.Client({ connectionString: url });
  await db.connect();
  try {
    // Waiting for the store lock while another producer writes is correct; a
    // transaction left open elsewhere would hold it indefinitely.
    await db.query("set lock_timeout = '5min'");
    const stats = await indexShard(db, {
      repo: REPO,
      modality: 'code',
      chunks,
      provider: voyageProvider({ apiKey, model: CODE_MODEL }),
      concurrency: 2,
      maxShrink: process.env.ALLOW_SHRINK === '1' ? undefined : MAX_SHRINK,
      sourceRev: process.env.GITHUB_SHA ?? null,
    });
    console.log(`${REPO}/code (${CODE_MODEL}): ${stats.total} chunks, ${stats.embedded} embedded, ${stats.unchanged} unchanged, ${stats.deleted} deleted`);
  } finally {
    await db.end();
  }
}

await main();
