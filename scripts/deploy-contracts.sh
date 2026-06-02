#!/usr/bin/env bash
set -euo pipefail

# DagLock contract deployment helper
# Compiles .sil files and extracts template hashes

SILVERC="${SILVERC_BIN:-silverc}"
CONTRACTS_DIR="${1:-contracts/src}"

echo "==> Compiling DagLock SilverScript contracts"
echo "    Source: ${CONTRACTS_DIR}"

for sil_file in "${CONTRACTS_DIR}"/*.sil; do
    name=$(basename "${sil_file}" .sil)
    echo ""
    echo "── Compiling ${name}.sil ──"
    
    # Compile to Kaspa script bytecode
    "${SILVERC}" "${sil_file}" --output "/tmp/${name}.bc" 2>&1
    
    # Extract template hash (BLAKE2b-160 of compiled bytecode prefix)
    # This is a placeholder — exact tooling depends on SilverScript CLI output
    echo "    Compiled: /tmp/${name}.bc"
    
    if command -v b2sum &>/dev/null; then
        template_hash=$(b2sum -l 160 "/tmp/${name}.bc" | cut -d' ' -f1)
        echo "    Template hash: ${template_hash}"
    else
        echo "    Install b2sum (from coreutils) to compute template hash"
    fi
done

echo ""
echo "==> Done."
