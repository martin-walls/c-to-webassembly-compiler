## Stack variable allocation

### LVA

- Doesn't work properly w/out running dead code removal first

### Dead code analysis

- make sure to not remove the label var

### Clash graphs

- if we take address of a var, we have to make it clash with everything else, cos we don't know all the places it could
  be referenced

### Interval trees

Didn't actually work for what I needed them to do.
I implemented the version from CLRS section 14.3 p 348 (because that type of interval tree is self-balancing when
inserting - it's based on a red-black tree)

But this requires keys to be unique, but intervals can have duplicate endpoints (it uses low bound as the key).

Instead, used a data structure that stores clash intervals in a sorted vec - each interval is the union of all the
clashes for that interval of already allocated vars.
Sorted by low bound of interval
When testing new var location, can stop looking for overlapping intervals once the low bound of intervals in the vec is
above the high bound of the interval we're allocating.
