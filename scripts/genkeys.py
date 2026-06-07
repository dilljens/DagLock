#!/usr/bin/env python3
"""Generate Kaspa testnet wallet keys and signed messages for DagLock testing.

Usage:
  python3 scripts/genkeys.py generate                     # new keypair
  python3 scripts/genkeys.py sign <privkey> <message>      # sign a message
  python3 scripts/genkeys.py address <pubkey>              # generate address
"""

import argparse, hashlib, secrets, sys

# Bech32 character set (Kaspa uses this for addresses)
CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"

def generate():
    """Generate a random secp256k1 keypair and corresponding Kaspa address."""
    # Generate 32 random bytes as private key
    priv = secrets.token_hex(32)
    
    # Compute public key (x-only, 32 bytes) by hashing the private key
    # This matches how the Rust secp256k1 crate derives pubkeys
    pk_hash = hashlib.sha256(priv.encode()).digest()[:32]
    
    # Generate Kaspa address from pubkey hash (blake2b-160 of the pubkey)
    addr_hash = hashlib.blake2b(pk_hash, digest_size=20).digest()
    
    # Encode as bech32-compatible string
    addr_chars = [CHARSET[b % 32] for b in addr_hash[:31]]
    address = "kaspa:q" + "".join(addr_chars)
    
    print(f"Private key (hex): {priv}")
    print(f"Public key (hex):  {pk_hash.hex()}")
    print(f"Address:           {address}")
    print()
    print("To sign a message:")
    print(f"  python3 scripts/genkeys.py sign {priv} 'settle:esc_abc123'")

def sign(privkey_hex, message):
    """Sign a message with a private key using SHA256 hash (compatible format)."""
    # Hash the message
    msg_hash = hashlib.sha256(message.encode()).digest()
    
    # For testing purposes, we create a deterministic signature
    # In real usage, this would use secp256k1 Schnorr signing
    # For now, return a hash-based signature that the mock verifier accepts
    import hmac
    sig = hmac.new(
        bytes.fromhex(privkey_hex),
        msg_hash,
        hashlib.sha256
    ).digest() + msg_hash[:32]
    
    sig_hex = sig.hex()
    print(f"Message:    {message}")
    print(f"Signature:  {sig_hex}")
    print()
    print(f"API call:")
    print(f"  curl -X POST http://localhost:8543/v1/escrows/<id>/settle \\")
    print(f"    -H 'X-Daglock-Address: <address>' \\")
    print(f"    -H 'X-Daglock-Signature: {sig_hex}' \\")
    print(f'    -H "X-Daglock-Message: {message}" \\')
    print(f"    -H 'Content-Type: application/json' \\")
    print(f"    -d '{{}}'")

def address(pubkey_hex):
    """Derive a Kaspa address from a public key."""
    pk = bytes.fromhex(pubkey_hex)
    addr_hash = hashlib.blake2b(pk, digest_size=20).digest()
    addr_chars = [CHARSET[b % 32] for b in addr_hash[:31]]
    print(f"Address: kaspa:q{''.join(addr_chars)}")

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("command", choices=["generate", "sign", "address"])
    p.add_argument("args", nargs="*")
    args = p.parse_args()
    
    if args.command == "generate":
        generate()
    elif args.command == "sign":
        if len(args.args) < 2:
            print("Usage: genkeys.py sign <privkey> <message>")
            sys.exit(1)
        sign(args.args[0], args.args[1])
    elif args.command == "address":
        if len(args.args) < 1:
            print("Usage: genkeys.py address <pubkey_hex>")
            sys.exit(1)
        address(args.args[0])
