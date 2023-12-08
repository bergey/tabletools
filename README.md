# tabletools

A collection of command-line programs for working with semi-structured data.

This is very new software, and the names of command-line flags will likely change between versions.

## unjustify

### examples

Docker output is easy to read, but particularly annoying to work with programatically, because both data rows and the `Container ID` header contain spaces.  With `--whitespace double`, `unjustify` correctly separates columns.

`docker ps | unjustify --whitespace=double`

### --help

```
make tables formatted for human readers easier to program against

By default handles justified whitespace-separated columns, including many cases where individual fields include whitespace.

Usage: unjustify [OPTIONS] [OUTPUT_COLUMNS]...

Arguments:
  [OUTPUT_COLUMNS]...
          

Options:
  -i, --insensitive
          case insensitive match for column names

  -d, --delimiters <DELIMITERS>
          additional column delimiters
          
          [default: ]

  -w, --whitespace <WHITESPACE>
          whitespace delimited?
          
          [default: any]
          [possible values: any, double, ignore]

  -b, --border
          count +-| and other border drawing characters as delimiters

  -O, --output <OUTPUT_DELIMITER>
          output delimiter (default ,)

      --unit-separator
          ascii unit separator character (overrides output delimiter)

      --line-delimiter <LINE_DELIMITER>
          line delimiter (default newline)

      --record-separator
          ascii record separator character (overrides line delimiter)

  -0
          null (overrides line delimiter)

  -H, --header
          pick columns from first row only

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## unnest

### examples

The example below sets the output delimiter to `|` so it is visually distinguished from the `.` in `publisher.name` and does not conflict with fields that include spaces.  It shows:
- multiple rows for objects containing lists
- accepting number values (date)
- fields present in only some records (subtitle)
- fields which may contain single values or arrays (authors)

```
$ unnest -- -O\| < examples/books.json

authors|date|publisher.locations|publisher.name|subtitle|title
Daniel Jackson|2012|Cambridge, Massachusetts|MIT Press|Logic, Language, and Analysis|Software Abstractions
Daniel Jackson|2012|London, England|MIT Press|Logic, Language, and Analysis|Software Abstractions
Brian W Kernighan|1984|Murray Hill, New Jersey|Bell Laboratories||UNIX Programming Environment, The
Rob Pike|1984|Murray Hill, New Jersey|Bell Laboratories||UNIX Programming Environment, The
Benjamin C. Pierce|2002|Cambridge, Massachusetts|MIT Press||Types and Programming Languages
Benjamin C. Pierce|2002|London, England|MIT Press||Types and Programming Languages
Mor Harchol-Balter|2013||Cambridge University Press|Queuing Theory in Action|Performance Modeling and Design of Computer Systems
```

### --help

```
turn nested JSON into tables

Usage: unnest [OPTIONS]

Options:
  -O, --output-delimiter <OUTPUT_DELIMITER>
          between columns of output [default single space]
      --line-delimiter <LINE_DELIMITER>
          between lines of output [default newline]
      --attribute-separator <ATTRIBUTE_SEPARATOR>
          in column names, between nested json object keys [default: .]
      --missing <MISSING>
          output representation of missing values [default: ]
  -h, --help
          Print help
  -V, --version
          Print version
```
