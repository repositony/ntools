# Notes on the MCTAL file format

This `readme` contains notes for the formatting of the MCNP6 MCTAL data for
reference.

The full crate documentation is published [here](https://repositony.github.io/ntools/ntools_mctal/index.html).

## Header block

Step-by-step through the file, the first line contains:

`<code> <version> <probid> <dump> <nps> <nrn>`

```text
6.2 => A8    A8 A19 I15    I15 1x I15
6.3 => A8 3x A5 A19 I11 1x I20 1x I15
6.3 => A8 3x A5 A19 I5  1x I15 1x I15
```

The message is the content of the next line, starting with a blank space.

`<message>`

```text
6.2 => 1x A79
6.3 => 1x A79
```

The number of tallies and number of perturbations have keyworks. Perturbation
key/value is optional and not included if none are defined in the run.

`NTAL <n> NPERT <m>`

```text
6.2 => A4 I6  1X A5 I6
6.3 => A5 I11    A6 I11
6.3 => A5 I5     A6 I5
```

Finally there is a list of tally numbers, collected over multiple lines and
splitting every 16 values.

`<tally numbers>...`

```text
6.2 => 16I5
6.3 => 16I5
```

## Tally block

The tally can be any standard `F` tally.

`TALLY <id> <i> <j> <k>`

```text
6.2 => A5 I5*     I2 2I5
6.3 => A5 I42 T20 I2 2I5
```

These correspond to the tally id followed by:

- `<i>` describes particle types
  - `<i>` > 0 => 1=N, 2=P, 3=N+P, 4=E, 5=N+E, 6=P+E, 7=N+P+E
  - `<i>` < 0 => number of particle types on the next line
    - next line `<particle numbers>...`
- `<j>` is the detector tally type
  - 0 = None
  - 1 = Point
  - 2 = Ring
  - 3 = Pinhole
  - 4 = TransmittedRectangular
  - 5 = TransmittedCylindrical
- `<k>` is the tally modifier
  - 0 = None
  - 1 = Star
  - 2 = Plus

If `<i>` is negative then the next line contains a list of 0 or 1 for each of
the 37 particle types.

`<particle numbers>...`

```text
6.2 => 40I2
6.3 => 40I2
```

Tally comments can span multiple lines, and these are stored on as many lines as
needed, starting with 5 blanks.

`<comment line>...`

```text
6.2 => 5x A75
6.3 => 5x A75
```

### Tally bin information

#### Overview of bins

Bin information is stored under a series of character tags. For reference these
are:

A full quick-reference is:

| Token | Tag     | Flag    | Unbound | List                      |
| ----- | ------- | ------- | ------- | ------------------------  |
| f     | &cross; | &check; | &cross; | &check; (unless detector) |
| d     | &cross; | &check; | &cross; | &cross;                   |
| u     | &check; | &check; | &cross; | &check; (if special)      |
| s     | &check; | &check; | &cross; | &check; (if segment)      |
| m     | &check; | &check; | &cross; | &cross;                   |
| c     | &check; | &check; | &check; | &check;                   |
| e     | &check; | &check; | &check; | &check;                   |
| t     | &check; | &check; | &check; | &check;                   |

Where the tokens are:

| Token | Description                                     |
| ----- | ----------------------------------------------- |
| f     | regions (i.e. cell, surface, or detector bins)  |
| d     | flagged bins                                    |
| u     | user bins                                       |
| s     | segments                                        |
| m     | multiplier bins                                 |
| c     | cosine bins                                     |
| e     | energy bins                                     |
| t     | time bins                                       |

Note that if the value is `0` this means the bin is unbounded.

Some tokens are followed by values under certain conditions, and may also have
multiple character tags to indicate binning strategy.

For example, on the energy bins:

| Tag example | Description                                 |
| ----------- | ------------------------------------------- |
| e           | one unbounded energy bin (n=0 instead of 1) |
| et          | includes a "`t`otal" bin                    |
| ec          | energy bins are `c`umulative                |

While MCNP does not print this, flags are allowed in user generated mctal files.
When the flag is `0` or `none`, values are upper bin boundaries. For anything
else, values are points where the tally values should be plotted.

For example:

| Flag example | Description                                   |
| ------------ | --------------------------------------------- |
| c 10         | 10 cosine bins, no flag, assumed upper bounds |
| c 10 0       | 10 cosine bins, upper bounds                  |
| c 10 1       | 10 cosine bins, discrete points               |

#### [f] Region bins

`F <n>`, where `n` is the number of regions

```text
6.2 => A2    I8
6.3 => A2 1x I7
```

The cell, surface, or detector numbers are then split every 11 integer values.
Note that `0` means that the cell, surface, or detector bin is made up of
several, and the user must know what they wrote in the input file. This is why
a hashmap (like a python dictionary) can not be used with these as keys.

`<numbers>...`

```text
6.2 => 11I7
6.3 => 11I7
```

#### [d] Flagged bins

Flagged bins are the number of total vs. direct or flagged vs. unflagged bins.
Note that:

- detectors - n=2 unless there is ND on the F5 card
- cell/surface tallies = n=1 unless there is an SF or CF card.

`D <n>`

```text
6.2 => A2    I8
6.3 => A2 1x I7
```

#### [u, ut, uc] User bins

Number of user bins, including the total bin if there is one. This may be one of
three possible tags.

`U <n> / UT <n> / UC <n>`

```text
6.2 => A2    I8
6.3 => A2 1x I7
```

Ok and while missing from the manual, you can actually have bins listed here

##### Problem edge case

```text
f1:p  3.3
e1    1.0 100.0
t1    0.001e8 60e8
ft1   scx 1 roc 10648648
tf1   1 1 1 1 1 1 2 2   1 1 2 1 1 1 2 2 $ signal bins, noise bins
```

gives

```text
f        1
    3.3
d        1
ut       3   <--- notice no bloddy bins listed for special FT/TF cards
s        0
m        0
c        0
et       3
  1.00000E+00  1.00000E+02
tt       3
  1.00000E+05  6.00000E+09
```

#### [s, st, sc] Segment bins

Number of segment bins. This may be one of three possible tags.

If the tally is a radiograph tally, then a list of the bin boundaries will be printed.

`S <n> / ST <n> / SC <n>`

```text
6.2 => A2    I8
6.3 => A2 1x I7
```

#### [m, mt, mc] Multiplier bins

Number of multiplier bins. This may be one of three possible tags.

`M <n> / MT <n> / MC <n>`

```text
6.2 => A2    I8
6.3 => A2 1x I7
```

#### [c, ct, cc] Cosine bins

Cosine bins. This may be one of three possible tags.

`C <n> <f> / CT <n> <f> / CC <n> <f>`

where,

- `<n>` is the number of cosine bins
- `<f>` is an integer flag
  - `f`=0 or none     => cosine values are boundaries
  - `f`=anything else => points where the tally values should be plotted

```text
6.2 => A2    I8 I4
6.3 => A2 1x I7 I4
```

List of cosine values if appropriate, on as many lines as necessary.

`<cosine values>...`

```text
6.2 => 6ES13.5
6.3 => 6ES13.5
```

#### [e, et, ec] Energy bins

Energy bins. This may be one of three possible tags.

`E <n> <f> / ET <n> <f> / EC <n> <f>`

where,

- `<n>` is the number of cosine bins
- `<f>` is an integer flag
  - `f`=0 or none     => energy values are boundaries
  - `f`=anything else => points where the tally values should be plotted

```text
6.2 => A2    I8 I4
6.3 => A2 1x I7 I4
```

List of energy bins if appropriate, on as many lines as necessary.

`<energy values>...`

#### [t, tt, tc] Time bins

Time bins. This may be one of three possible tags.

`T <n> <f> / TT <n> <f> / TC <n> <f>`

where,

- `<n>` is the number of cosine bins
- `<f>` is an integer flag
  - `f`=0 or none     => time values are boundaries
  - `f`=anything else => points where the tally values should be plotted

```text
6.2 => A2    I8 I4
6.3 => A2 1x I7 I4
```

List of time bins if appropriate, on as many lines as necessary.

`<time values>...`

### Tally results

All results are dumped into a single list of `value`/`error` pairs.

The order of the values is that of a 9-dimensional Fortran array. If it were
dimensioned (2, NT, NE, ..., NF) where NT is the number of time bins, NE is the
number of energy bins, etc..., and NF is the number of cell, surface, or
detector bins.

In other words, time bins are under energy bins, which are under cosine bins,
and so on.

`vals <values>`

```text
6.2 => A4 4(ES13.5 F7.4)
6.3 => A4 4(ES13.5 F7.4)
```

or, if there are tally perturbations

`vals pert <values>...`

```text
6.2 => A10 4(ES13.5 F7.4)
6.3 => A10 4(ES13.5 F7.4)
```

### Tally fluctuation chart

The entire tally fluctuation chart is recorded. The first line contains the
number of bins and metadata.

`TFC n jtf`

- `n` is the number of sets of tally fluctuation data
- `jtf` is a list of 8 numbers, indices of the tally fluctuation chart bins

```text
6.2 => A3 I5  8I8
6.3 => A4 I4  8(1x,I7)
6.3 => A4 I11 8(1x,I11)
```

The records for the table are recorded as the following, on as many lines as
needed:

`<nps> <mean> <error> <fom>`

```text
6.2 =>    I15 3ES13.5
6.3 => 1x I14 3ES13.5
6.3 => 1x I20 3ES13.5
```

## KCODE block

During a KCODE problem this block will be written. There is some general
information in the first line, followed by the result of each cycle.

`kcode <kcz> <ikz> <l>`

- `kcz` is the number of recorded KCODE cycles
- `ikz` is the number of settle cycles
- `l` is the number of variables provided for each cycle

```text
6.2 => A5 3I5
6.2 => A5 3I10
6.3 => A5 3I5
6.3 => A5 3I10
```

List cycle results, on as many lines as needed. Note that this may be 18 or 19
values long. The Figure of Merit is only printed when `mct` is used on the
`PRDMP` card.

`<values>...`

```text
6.2 => 5ES12.6
6.3 => 5ES12.6
```

## TMESH block

The TMESH block is the old MCNPX mesh tally format. For whatever reason, the
TMESH data are also written to the MCTAL file. These are also known in user manuals as "Superimposed Mesh Tally Type A".

The format is very similar to the general Tally data block, though a lot of the
tags are repurposed.

While not documented, MCNP6.2 also allows writing TMESH to MCTAL files. It is
assumed that the 6.3 formatting applies to 6.2 until I find otherwise.

### Tmesh header info

Note the reuse of the `tally` tag. The negatives are not used for anything other
than makring it as a tmesh over the normal tally.

`tally nugd < -j > < -j8 >`

- `nugd` is the mesh tally number
- `j` is the number of particles in the mesh tally
- `j8` is the mesh type:
  - 1 rectangular
  - 2 cylindrical
  - 3 spherical

```text
6.3 => A5 3I5
```

If `<i>` is negative then the next line contains a list of 0 or 1 for each of
the 37 particle types.

`<particle numbers>...`

```text
6.2 => 40I2
6.3 => 40I2
```

### Tmesh geometry bounds

The tmesh bounds are printed, and are pretty much the FMESH equivalents. For
example, `ng1` == iints and `cora` == imesh values.

`f mxgc 0 ng1 ng2 ng3`

- `mxgc` is the total number of voxels
- `ng1` is the number of bins on the CORA card
- `ng2` is the number of bins on the CORB card
- `ng3` is the number of bins on the CORC card

```text
6.3 => A2 I8 4I5
```

All mesh geometry bounds are then listed over as many lines as needed.

`<values>...`

```text
6.3 => 6ES13.5
```

### Tmesh bin counts

The same markers in the same order that Tally uses, except many are unused and should be 1. These include:

- `d` 1 (A2 I8)
- `u` 1 (A2 I8)
- `m` 1 (A2 I8)
- `c` 1 (A2 I8)
- `e` 1 (A2 I8)
- `t` 1 (A2 I8)

The only special case is that of segmented tallies:

`s <mxgv>`

- `mxgv` is the number of S bins on the tally from different keywords. For example,
  - mxgv = 1 for `RMESH 1:P FLUX`
  - mxgv = 3 for `RMESH 3 TOTAL DE/DX RECOL`

```text
6.3 => A2 I8
```

### Tmesh voxel results

Like Tally, all results are dumped into a single list of `value`/`error` pairs.

While the ordering is the same there are fewer bins. In the event that there are
multiple S-bins, the CORA, CORB, and CORC coordinates for each S-bin are grouped.

The `kjis` ordering follows the FMESH mesh, where CORA bins are under the CORB
bins, which are under the CORC bins, which are under the S-bins.

`vals <values>`

```text
6.3 => A4 4(ES13.5 F7.4)
```
