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

## Combineren (Nesting)

Je kunt meerdere berekeningen in één keer doen door expressies te combineren. In programmeren wordt dit ook wel "Nesting" genoemd.

Een voorbeeld: `a = 10; b = a + 2 + 5`.

In Polder kun je het volgende schrijven:

```
Laat a 10 zijn.
Laat b de som van a en de som van 2 en 5 zijn.
```

Let op dat de binnenste expressie (`de som van 2 en 5`) als eerste wordt berekend, daarna de buitenste (`de som van 3 en {7}`).
In dit geval maakt de volgorde niet uit, maar in andere gevallen kan dat wel een verschil maken.
 
Wil je meer weten over de volgorde van berekenen en hoe je die volgorde kan veranderen? Zie [Voorrang](#voorrang-precedence).

Let op dat elke wiskundige expressie maar 2 waarden kan hebben - een linkse en een rechtse expressie. Dit voorbeeld is dus niet geldig in Polder:
```
Laat b de som van a en 2 en 5 zijn.
```

## Voorrang (Precedence)

In veel programmeertalen is het soms onduidelijk in welke volgorde berekeningen moeten worden gedaan. Bekijk het volgende voorbeeld:

```
3 + 4 * 5
```
Volgens de rekenregels moeten vermenigvudigingen en delingen eerst, daarna optellen en aftrekken. Het voorbeeld zou dus `23` moeten zijn (eerst `4 * 5 = 20`, dan `3 + 20 = 23`).
Echter, als je de formule van links naar rechts leest, zou je als antwoord `35` krijgen.

De manier waarop de meeste programeertalen bepalen in welke volgorde berekeningen worden gedaan, wordt ook wel "Operator precedence" genoemd. Deze gaan uit van de rekenregels, zodat het juiste antwoord altijd wordt berekend. Als je een andere volgorde van berekenen wilt gebruiken, moet je haakjes toevoegen aan de formule:

```
(3 + 4) * 5
```
Dit zal wel het antwoord `35` produceren.

Polder heeft geen concept van "Operator precedence", omdat Nederlandse grammatica dit al voor ons doet. Bekijk de volgende voorbeelden:

```
Druk de som van 3 en het product van 6 en 8 af.
```
Deze regel berekent eerst `6 * 8 = 48` en daarna `3 + 48 = 51`. Dit is de verwachte volgorde van rekenen.

Maar wat nou als je de berekening wél andersom zou willen doen (`(3 + 6) * 8`)?

Je hoeft alleen maar de volgorde van expressies om te draaien:

```
Druk het product van de som van 3 en 6 en 8 af.
```

Om wat duidelijker te laten zien wat hier gebeurt, hier een voorbeeld met haakjes (Geen geldige syntax in Polder!):

```
Druk (het product van (de som van 3 en 6) en 8) af.
```

Vergelijk dat met het eerste voorbeeld:
```
Druk (de som van 3 en (het product van 6 en 8)) af.
```

In het tweede voorbeeld wordt dus eerst `3 + 6 = 9` berekend, daarna `9 * 8 = 72`.
Op deze manier is het altijd duidelijk in welke volgorde berekeningen moeten worden gedaan.

## Verdere informatie

**_Let op_**: Bij een deeloperatie; indien een variabele geen kommagetal is, wordt het fractionele gedeelte van het getal weggegooid.
