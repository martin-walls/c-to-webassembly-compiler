
## LVA

- Doesn't work properly w/out running dead code removal first

## Dead code analysis

- make sure to not remove the label var

## Clash graphs

- if we take address of a var, we have to make it clash with everything else, cos we don't know all the places it could be referenced
