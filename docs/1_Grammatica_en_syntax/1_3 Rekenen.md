# Rekenen

Simpele berekeningen kunnen worden gedaan met constructies zoals `Tel x bij y op.` of `Vermenigvuldig x met 2.`.

In expressies wordt de zinsconstructie anders. Wanneer je een nieuwe variabele declareert en de waarde daarvan wil laten afhangen van een rekenkundige operatie, gebruik je de expressiesyntax.

De rekenkundige functies die beschikbaar zijn, worden hieronder uitgelegd met een vergelijkbare C-stijl syntax.

| Polder (Statement)            | Polder (Expressie)                           | C-stijl                           |
| ----------------------------- | -------------------------------------------- | --------------------------------- |
| `Tel {b} bij {a} op.`         | `Laat {x} de som van {a} en {b} zijn.`       | `{a} += {b}` of `{x} = {a} + {b}` |
| `Trek {b} van {a} af.`        | `Laat {x} het verschil van {a} en {b} zijn.` | `{a} -= {b}` of `{x} = {a} - {b}` |
| `Vermenigvuldig {a} met {b}.` | `Laat {x} het product van {a} en {b} zijn.`  | `{a} *= {b}` of `{x} = {b} * {a}` |
| `Deel {a} door {b}.`          | `Laat {x} het quotiënt van {a} en {b} zijn.` | `{a} /= {b}` of `{x} = {a} / {b}` |
| `Verhoog {a}.`                | _N.v.t._                                     | `{a}++` of `{a} += 1`             |
| `Verlaag {a}.`                | _N.v.t._                                     | `{a}--` of `{a} -= 1`             |

\* _P.S.: Een getal en een kommagetal kunnen momenteel nog niet bij elkaar opgeteld, van elkaar afgetrokken, met elkaar vermenigvuldigd of door elkaar gedeeld worden._ `{a}` _moet een kommagetal zijn om rekenkundige functies met een ander kommagetal te ondersteunen._

**_Let op_**: Bij een deeloperatie; indien een variabele geen kommagetal is, wordt het fractionele gedeelte van het getal weggegooid.
