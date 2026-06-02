// DagLock Indexer REST API client for the Telegram bot.

export class ApiClient {
  constructor(baseUrl) {
    this.baseUrl = baseUrl;
  }

  async request(path, options = {}) {
    const url = `${this.baseUrl}/v1${path}`;
    const res = await fetch(url, {
      headers: { 'Content-Type': 'application/json', ...options.headers },
      ...options,
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: { message: res.statusText } }));
      throw new Error(err.error?.message || `HTTP ${res.status}`);
    }
    return res.json();
  }

  getEscrow(id) {
    return this.request(`/escrows/${id}`);
  }

  listEscrows(address, status) {
    let path = `/escrows?address=${encodeURIComponent(address)}`;
    if (status) path += `&status=${status}`;
    return this.request(path);
  }

  listOffers() {
    return this.request('/offers');
  }

  getReputation(address) {
    return this.request(`/reputation/${encodeURIComponent(address)}`);
  }

  getReceipt(id) {
    return this.request(`/receipts/${encodeURIComponent(id)}`);
  }

  settleEscrow(id) {
    return this.request(`/escrows/${encodeURIComponent(id)}/settle`, { method: 'POST' });
  }

  refundEscrow(id) {
    return this.request(`/escrows/${encodeURIComponent(id)}/refund`, { method: 'POST' });
  }

  disputeEscrow(id, reason) {
    return this.request(`/escrows/${encodeURIComponent(id)}/dispute`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }

  cancelEscrow(id) {
    return this.request(`/escrows/${encodeURIComponent(id)}/cancel`, { method: 'POST' });
  }
}
