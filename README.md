# Jenesis contract templates

To use these contract templates:
1. Install Jenesis:
```bash
pip install jenesis
```
2. Create a project:
```bash
jenesis new my_project
```
3. Add the contract
```bash
jenesis add contract <template_name> my_contract
```

Currently available templates:
- **starter**: a bare-bones contract that sets and increments a counter
- **token**: a fungible token contract (Cosmwasm `cw20-base`)
- **proxy**: a basic proxy contract (Cosmwasm `cw1-subkeys`)
- **multisig**: a fixed multisig contract (Cosmwasm `cw3-fixed-multisig`)
- **nft**: a general token contract supporting nfts (Cosmwasm `cw1155-base`)



