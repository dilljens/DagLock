# Plan #3: Trade Bot + Offer Expiry Cleanup

**Goal:** Deploy the trade bot to populate the offer board, and enable server-side offer expiry enforcement so stale offers don't clutter the board.

**Effort:** 2-3 hours

---

## Phase 3A: Server-side offer expiry `[ ]`
**⏱ Timebox:** 1h

The query `reconcile_expired_offers` already exists in `indexer/src/db/queries/offers.rs`. I need to call it somewhere.

- [ ] Read `indexer/src/listener.rs` reconciliation loop (or main startup loop) and add a call to `queries::reconcile_expired_offers(&state.db).await?` that runs every 5 minutes
- [ ] If no periodic loop exists, add a simple tokio::spawn background task in `indexer/src/main.rs`:
  ```rust
  let db = pool.clone();
  tokio::spawn(async move {
      loop {
          tokio::time::sleep(Duration::from_secs(300)).await;
          if let Err(e) = queries::reconcile_expired_offers(&db).await {
              tracing::warn!("Failed to reconcile expired offers: {e}");
          }
      }
  });
  ```
- [ ] Check `expires_at` is set on offer creation (it may be null). If null, set a default (72h from creation).
- [ ] Verify the offer list endpoint filters out `expired` status by default

**✅ Checkpoint:** `curl /v1/offers?status=all` shows expired offers with different status after their expires_at passes

---

## Phase 3B: Deploy trade bot to VPS `[ ]`
**⏱ Timebox:** 1.5h

- [ ] The script `scripts/trade-bot.py` is already written with mock auth signing, two bot identities, and offer diversity (KAS/KRC20 pairs)
- [ ] Copy script to VPS: `rsync scripts/trade-bot.py root@VPS:/opt/daglock-trade-bot/trade-bot.py`
- [ ] Create systemd service: `/etc/systemd/system/daglock-trade-bot.service`
  ```ini
  [Unit]
  Description=DagLock Trade Bot
  After=network.target
  
  [Service]
  Type=oneshot
  ExecStart=/opt/daglock-trade-bot/trade-bot.py
  WorkingDirectory=/opt/daglock-trade-bot
  Environment=API_URL=http://127.0.0.1:8443
  
  [Install]
  WantedBy=multi-user.target
  ```
- [ ] Create systemd timer: `/etc/systemd/system/daglock-trade-bot.timer`
  ```ini
  [Unit]
  Description=DagLock Trade Bot — every 10 minutes
  
  [Timer]
  OnUnitActiveSec=10min
  Unit=daglock-trade-bot.service
  
  [Install]
  WantedBy=timers.target
  ```
- [ ] Enable and start: `systemctl daemon-reload && systemctl enable --now daglock-trade-bot.timer`
- [ ] Verify: `journalctl -u daglock-trade-bot -f` shows trades happening

**✅ Checkpoint:** `/v1/offers` returns 5-12 offers from `@trader_alice` and `@trader_bob`

---

## Phase 3C: Monitor and tune `[ ]`
**⏱ Timebox:** 30min

- [ ] Check rate limiter isn't blocking the bot (30 req/min default, bot does ~15 calls per cycle)
- [ ] Verify bot identity linking works (Telegram handles on offers)
- [ ] Adjust `MIN_OFFERS`/`MAX_OFFERS` if board looks empty or too full
- [ ] Verify stale offers get cleaned (both by bot's OFFER_TTL and server-side expire)

**✅ Checkpoint:** Offer board shows healthy activity with varied prices and assets
