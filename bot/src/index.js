// DagLock Telegram Bot — Trustless escrow from your chat.
// Requires: BOT_TOKEN env var (from @BotFather)
//           INDEXER_URL env var (default: http://localhost:8443)
//
// Run: BOT_TOKEN=xxx node src/index.js

import { Bot, Keyboard, InlineKeyboard } from 'grammy';
import { ApiClient } from './lib/api.js';

const token = process.env.BOT_TOKEN;
if (!token) {
  console.error('BOT_TOKEN environment variable required');
  process.exit(1);
}

const apiUrl = process.env.INDEXER_URL || 'http://localhost:8443';
const api = new ApiClient(apiUrl);
const bot = new Bot(token);

// ── Commands ─────────────────────────────────────────────────────────

bot.api.setMyCommands([
  { command: 'start', description: 'Start the bot or claim an escrow' },
  { command: 'create', description: 'Create a new escrow' },
  { command: 'offers', description: 'Browse open offers' },
  { command: 'reputation', description: 'Check counterparty reputation <address>' },
  { command: 'status', description: 'Check escrow status <id>' },
  { command: 'receipt', description: 'Fetch a settlement receipt <id>' },
  { command: 'dispute', description: 'Dispute an escrow <id> <reason>' },
  { command: 'cancel', description: 'Cancel an escrow <id>' },
  { command: 'help', description: 'Show help' },
]);

bot.command('start', async (ctx) => {
  const payload = ctx.match;
  if (payload?.startsWith('claim_')) {
    const escrowId = payload.replace('claim_', '');
    return handleClaim(ctx, escrowId);
  }
  await ctx.reply(
    '🔒 *DagLock — Trustless Escrow on Kaspa*\n\n' +
    'I help you create and manage trustless escrows using Kaspa smart contracts.\n\n' +
    '_No one can steal your funds — not even me._\n\n' +
    'Commands:\n' +
    '/create — Create an escrow\n' +
    '/offers — Browse offers\n' +
    '/status <id> — Check escrow\n' +
    '/receipt <id> — Fetch receipt\n' +
    '/dispute <id> <reason> — Dispute escrow\n' +
    '/cancel <id> — Cancel escrow\n' +
    '/reputation <address> — Check reputation',
    { parse_mode: 'Markdown' }
  );
});

bot.command('create', async (ctx) => {
  await ctx.reply(
    '📝 *Create Escrow*\n\n' +
    'To create an escrow, use the CLI:\n' +
    '```\ndaglock-cli create --amount 5000 --counterparty <address>\n```\n\n' +
    'Or visit: https://daglock.com/create',
    { parse_mode: 'Markdown' }
  );
});

bot.command('offers', async (ctx) => {
  try {
    const data = await api.listOffers();
    const offers = data.offers || [];

    if (offers.length === 0) {
      return await ctx.reply('📭 No open offers right now.');
    }

    let msg = '📋 *Open Offers*\n\n';
    for (const o of offers.slice(0, 5)) {
      const amount = (o.amount_sompi / 1e8).toFixed(2);
      msg += `• *${o.side.toUpperCase()}* ${amount} ${o.base_asset} for ${o.quote_asset}\n`;
      msg += `  ID: \`${o.id}\`\n\n`;
    }
    if (offers.length > 5) msg += `_...and ${offers.length - 5} more_`;

    await ctx.reply(msg, { parse_mode: 'Markdown' });
  } catch (err) {
    await ctx.reply('❌ Could not fetch offers: ' + err.message);
  }
});

bot.command('status', async (ctx) => {
  const id = ctx.match?.trim();
  if (!id) return await ctx.reply('Usage: /status <escrow-id>');

  try {
    const data = await api.getEscrow(id);
    const amount = (data.amount_sompi / 1e8).toFixed(2);
    const created = new Date(data.created_at * 1000).toISOString().slice(0, 19).replace('T', ' ');

    await ctx.reply(
      `📋 *Escrow: ${id}*\n\n` +
      `Status: \`${data.status}\`\n` +
      `Amount: ${amount} KAS\n` +
      `Buyer: \`${data.buyer_address.slice(0, 16)}...\`\n` +
      `Created: ${created} UTC` +
      (data.dispute_reason ? `\nReason: ${data.dispute_reason}` : ''),
      { parse_mode: 'Markdown' }
    );
  } catch (err) {
    await ctx.reply('❌ Escrow not found or API error: ' + err.message);
  }
});

bot.command('receipt', async (ctx) => {
  const id = ctx.match?.trim();
  if (!id) return await ctx.reply('Usage: /receipt <escrow-id>');

  try {
    const receipt = await api.getReceipt(id);
    await ctx.reply(
      `🧾 *Receipt*\n\n` +
      `ID: \`${receipt.receipt_id}\`\n` +
      `Status: \`${receipt.status}\`\n` +
      `Asset: ${receipt.asset}\n` +
      `Amount: ${receipt.amount_sompi} units` +
      (receipt.dispute_reason ? `\nReason: ${receipt.dispute_reason}` : ''),
      { parse_mode: 'Markdown' }
    );
  } catch (err) {
    await ctx.reply('❌ Receipt not found or API error: ' + err.message);
  }
});

bot.command('dispute', async (ctx) => {
  const [id, ...reasonParts] = (ctx.match || '').trim().split(/\s+/);
  const reason = reasonParts.join(' ').trim();
  if (!id || !reason) return await ctx.reply('Usage: /dispute <escrow-id> <reason>');

  try {
    const result = await api.disputeEscrow(id, reason);
    await ctx.reply(`⚠️ Escrow disputed: ${result.escrow_id}\nReason: ${reason}`);
  } catch (err) {
    await ctx.reply('❌ Could not dispute escrow: ' + err.message);
  }
});

bot.command('cancel', async (ctx) => {
  const id = ctx.match?.trim();
  if (!id) return await ctx.reply('Usage: /cancel <escrow-id>');

  try {
    const result = await api.cancelEscrow(id);
    await ctx.reply(`🛑 Escrow cancelled: ${result.escrow_id}`);
  } catch (err) {
    await ctx.reply('❌ Could not cancel escrow: ' + err.message);
  }
});

bot.command('reputation', async (ctx) => {
  const address = ctx.match?.trim();
  if (!address) return await ctx.reply('Usage: /reputation <kaspa-address>');

  try {
    const rep = await api.getReputation(address);
    const volume = (rep.total_volume_sompi / 1e8).toFixed(2);

    await ctx.reply(
      `📊 *Reputation*\n\n` +
      `Address: \`${address.slice(0, 16)}...\`\n` +
      `Trades: ${rep.trade_count}\n` +
      `Settled: ${rep.settled_count}\n` +
      `Refunded: ${rep.refunded_count}\n` +
      `Disputed: ${rep.disputed_count}\n` +
      `Dispute Rate: ${(rep.dispute_rate * 100).toFixed(1)}%\n` +
      `Refund Rate: ${(rep.refund_rate * 100).toFixed(1)}%\n` +
      `Score: ${rep.score.toFixed(2)}/5\n` +
      `Volume: ${volume} KAS`,
      { parse_mode: 'Markdown' }
    );
  } catch (err) {
    await ctx.reply('❌ Error: ' + err.message);
  }
});

bot.command('help', async (ctx) => {
  await ctx.reply(
    '🔒 *DagLock Bot Help*\n\n' +
    '/create — Create escrow (opens web interface)\n' +
    '/offers — Browse open offers\n' +
    '/status <id> — Check escrow status\n' +
    '/receipt <id> — Fetch receipt\n' +
    '/dispute <id> <reason> — Dispute escrow\n' +
    '/cancel <id> — Cancel escrow\n' +
    '/reputation <address> — Check reputation\n' +
    '/start <claim_ID> — Claim an escrow from a link\n' +
    '/help — This message',
    { parse_mode: 'Markdown' }
  );
});

// ── Claim handler ────────────────────────────────────────────────────

async function handleClaim(ctx, escrowId) {
  try {
    const data = await api.getEscrow(escrowId);
    const amount = (data.amount_sompi / 1e8).toFixed(2);

    const keyboard = new InlineKeyboard()
      .url('🔓 Claim via Browser', `https://daglock.com/claim/${escrowId}`);

    await ctx.reply(
      `🔓 *Claim Escrow*\n\n` +
      `You have been offered an escrow:\n\n` +
      `Amount: ${amount} KAS\n` +
      `Escrow: \`${escrowId}\`\n\n` +
      `To claim, open in browser and sign with your wallet:`,
      { parse_mode: 'Markdown', reply_markup: keyboard }
    );
  } catch (err) {
    await ctx.reply('❌ Could not load escrow: ' + err.message);
  }
}

// ── Start the bot ────────────────────────────────────────────────────

bot.start({ drop_pending_updates: true });
console.log(`DagLock Bot running... (indexer: ${apiUrl})`);
