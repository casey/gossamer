media
=====

todo:
- apps/comic show an error if it's the wrong kind of manifest
- add tests
- throw an error if package output is inside of package root

later:
- comic metadata:
  - title
  - isbn
  - series
  - volume
  - credits
  - original language
  - translation language
  - source is digital or scan
- comic checks:
  - files must all be same format
  - files must all be same size
  - allow double pages
  - check that files are actually JPEGs
    - check magic number
    - check that they deserialize
