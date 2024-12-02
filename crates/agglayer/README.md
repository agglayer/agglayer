# `agglayer`
```text
Agglayer command line interface

Usage: agglayer <COMMAND>

Commands:
  run              
  config           
  validate-config  
  prover-config    
  prover
```
## `agglayer` `run`
```text
Usage: agglayer run [OPTIONS]

Options:
  -c, --cfg <CFG>
          The path to the configuration file
          
          [env: CONFIG_PATH=]
          [default: agglayer.toml]
```
## `agglayer` `config`
```text
Usage: agglayer config --base-dir <BASE_DIR>

Options:
  -b, --base-dir <BASE_DIR>
          The path to the agglayer dir
          
          [env: CONFIG_PATH=]
```
## `agglayer` `validate-config`
```text
Usage: agglayer validate-config --path <PATH>

Options:
  -p, --path <PATH>
          The path to the agglayer dir
```
## `agglayer` `prover-config`
```text
Usage: agglayer prover-config
```
## `agglayer` `prover`
```text
Usage: agglayer prover [OPTIONS]

Options:
  -c, --cfg <CFG>
          The path to the configuration file
          
          [env: PROVER_CONFIG_PATH=]
          [default: agglayer-prover.toml]
```