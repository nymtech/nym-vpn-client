// Writes the nym-vpn-client code and docs shards to the public retrieval store,
// which the Nym docs MCP route searches alongside the nym shards (search_code and
// search_docs). The docs shard is the repository's own Markdown files. The store,
// its roles and its CI setup are described in the nym repository's
// documentation/MCP-STORE.md.
//
// Run from .github/indexer:
//   node index.mts --dry-run       walk, chunk and check the gates; write nothing
//   RETRIEVAL_STORE_WRITE_URL=... RETRIEVAL_STORE_CA_CERT="$(cat ca.pem)" VOYAGE_API_KEY=... node index.mts
//
// RETRIEVAL_SOURCE_DIR is the checkout to index (default: this repository) and
// RETRIEVAL_SOURCE_REF the git ref its links point at (default: develop). The
// store write indexes a release: the tag's checkout, with links at the tag.
// RETRIEVAL_SOURCE_REV is the commit recorded on the shards.
//
// RETRIEVAL_STORE_CA_CERT is the PEM text of the CA that signed the store's
// certificate. The URL then carries no sslmode or sslrootcert (storeConnection).
//
// The store skips a chunk whose id and content hash it already holds, so a run
// embeds only the chunks that changed since the last one. The walks read the
// working tree, so untracked files in a local checkout are included.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import pg from 'pg';
import { chunkPages } from '@nymproject/retrieval-indexer/chunker';
import { chunkCodeFile } from '@nymproject/retrieval-indexer/code-chunker';
import { voyageProvider } from '@nymproject/retrieval-indexer/embed';
import { indexShard, storeConnection } from '@nymproject/retrieval-indexer/store';
import { assertNoSilentDrops, assertRootsYieldChunks, collectMarkdownPages, collectSourceFiles } from '@nymproject/retrieval-indexer/walk';

const REPO = 'nym-vpn-client';
const REPO_ROOT = path.resolve(process.env.RETRIEVAL_SOURCE_DIR || path.join(path.dirname(fileURLToPath(import.meta.url)), '../..'));
const GITHUB = `https://github.com/nymtech/${REPO}/blob/${process.env.RETRIEVAL_SOURCE_REF || 'develop'}`;
// wireguard holds the Nym-maintained Go netstack and forwarders (libwg).
// nym-vpn-windows is C++, which the package does not chunk, so it would add no chunks.
const ROOTS = ['nym-vpn-core', 'nym-vpn-app', 'nym-vpn-android', 'nym-vpn-apple', 'wireguard'];
// The models the MCP route embeds queries with; one model per modality.
const CODE_MODEL = 'voyage-code-3';
const DOCS_MODEL = 'voyage-3-large';
// A run that would delete more than this share of the shard is refused, so a
// broken walk cannot empty part of the public index. ALLOW_SHRINK=1 lets an
// intended large removal through.
const MAX_SHRINK = 0.2;
// The probe file that the post-write step and retrieval-indexer's mcp-checks run
// against the live MCP server. Each root needs a path probe.
const PROBE_FILE = path.join(path.dirname(fileURLToPath(import.meta.url)), 'mcp-probes.json');

function assertRootsHaveProbes(): void {
  const { probes } = JSON.parse(readFileSync(PROBE_FILE, 'utf-8')) as { probes: { path?: { repo: string; path: string } }[] };
  const paths = probes.filter((p) => p.path?.repo === `nymtech/${REPO}`).map((p) => p.path!.path);
  const under = (p: string, root: string) => p === root || p.startsWith(`${root}/`);
  const unprobed = ROOTS.filter((root) => !paths.some((p) => under(p, root)));
  const orphaned = paths.filter((p) => !ROOTS.some((root) => under(p, root)));
  const problems = [
    unprobed.length ? `add a path probe for ${unprobed.join(', ')}` : '',
    orphaned.length ? `these probes point outside ROOTS: ${orphaned.join(', ')}` : '',
  ].filter(Boolean);
  if (problems.length) throw new Error(`mcp-probes.json: ${problems.join('; ')}`);
}

async function main(): Promise<void> {
  const dryRun = process.argv.includes('--dry-run');
  assertRootsHaveProbes();

  const { files, oversized, dropped } = collectSourceFiles(REPO_ROOT, ROOTS);
  assertNoSilentDrops(dropped);
  const codeChunks = (
    await Promise.all(
      files.map((f) => chunkCodeFile(f.content, f.path, { repo: REPO, source: `${REPO}-code`, github: GITHUB })),
    )
  ).flat();
  assertRootsYieldChunks(ROOTS, files, codeChunks.map((c) => c.title));
  const accepted = Object.entries(dropped).map(([ext, f]) => `.${ext} ${f.length}`).join(', ');
  console.log(`${files.length} source files in ${ROOTS.length} roots -> ${codeChunks.length} code chunks (not chunked, accepted: ${accepted || 'none'}; oversized: ${oversized.length})`);

  const pages = collectMarkdownPages(REPO_ROOT, ['.'], { source: `${REPO}-docs`, github: GITHUB });
  const docChunks = chunkPages(pages, { siteUrl: GITHUB, repo: REPO });
  console.log(`${pages.length} Markdown files -> ${docChunks.length} docs chunks`);
  if (dryRun) return;

  const url = process.env.RETRIEVAL_STORE_WRITE_URL;
  const apiKey = process.env.VOYAGE_API_KEY;
  if (!url || !apiKey) throw new Error('index: set RETRIEVAL_STORE_WRITE_URL and VOYAGE_API_KEY, or pass --dry-run');

  // One client, not a pool: a write run is one transaction on one connection.
  const db = new pg.Client(storeConnection(url, process.env.RETRIEVAL_STORE_CA_CERT));
  await db.connect();
  try {
    // Waiting for the store lock while another producer writes is correct; a
    // transaction left open elsewhere would hold it indefinitely.
    await db.query("set lock_timeout = '5min'");
    // Each shard is its own transaction: a failed docs write leaves the code shard written.
    const shards = [
      { modality: 'code', chunks: codeChunks, model: CODE_MODEL },
      { modality: 'docs', chunks: docChunks, model: DOCS_MODEL },
    ] as const;
    for (const { modality, chunks, model } of shards) {
      const stats = await indexShard(db, {
        repo: REPO,
        modality,
        chunks,
        provider: voyageProvider({ apiKey, model }),
        concurrency: 2,
        maxShrink: process.env.ALLOW_SHRINK === '1' ? undefined : MAX_SHRINK,
        sourceRev: process.env.RETRIEVAL_SOURCE_REV || process.env.GITHUB_SHA || null,
      });
      console.log(`${REPO}/${modality} (${model}): ${stats.total} chunks, ${stats.embedded} embedded, ${stats.unchanged} unchanged, ${stats.deleted} deleted`);
    }
  } finally {
    await db.end();
  }
}

await main();
