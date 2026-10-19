# Variabelen

## Variabelen maken

Variabelen worden gemaakt met de conventie `Laat <var_naam> <type?> <waarde?> zijn.`

Eerst geef je de naam aan de variabele. Vervolgens wijs je een [type](#datatypes) toe (optioneel). Daarna geef je (optioneel) de waarde op.

```
Laat x het getal 42 zijn.
```

maakt een variabele `x` aan, van het type `getal` en met de waarde `42`.

---

Je kunt een variabele zonder waarde opgeven en deze later een waarde [toewijzen](#toewijzen). Het lidwoord wat je zou gebruiken om het type aan te geven, verandert dan van `de`/`het` naar `een`:

```
Laat groet een tekst zijn.
```

maakt een variabele `groet` aan, van het type `tekst`, zonder waarde.

---

Je kunt ook het type weglaten als je een initiele waarde opgeeft:

```
Laat naam "Jan" zijn.
```

Polder maakt hier automatisch een variabele `naam` met type `tekst` en waarde `"Jan"` van.

Een variabele moet óf een type, óf een initiele waarde bevatten. `Laat variabele zijn.` is geen geldige zin in Polder.

### Constante waardes

Als je wilt voorkomen dat een variabele wordt aangepast in het vervolg (De waarde is dan 'constant'), voeg dan `voorgoed` in vóór het type.

```
Laat taal voorgoed de tekst "Polder" zijn.
```

Constante variabelen moeten een initele waarde hebben. `Laat taal voorgoed een tekst zijn.` zal een foutmelding opleveren.

### Voorbeelden van variabelen maken:

Een aantal voorbeelden van het maken van variabelen, met een vergelijking in C-stijl talen:

| C-stijl                        | Polder                                       |
| ------------------------------ | -------------------------------------------- |
| `string invoer;`               | `Laat invoer een tekst zijn.`                |
| `int x = 5;`                   | `Laat x het getal 5 zijn.`                   |
| `auto naam = "Jan";`           | `Laat naam "Jan" zijn.`                      |
| `const string taal = "Polder"` | `Laat taal voorgoed de tekst "Polder" zijn.` |

## Expressies

Het maken van variabelen kan met een vaste waarde zoals hierboven beschreven, maar het is ook mogelijk om een expressie te gebruiken. Daarmee kun je de waarde van een variabele af laten hangen van een andere variabele, of een berekening (zie bijvoorbeeld [Rekenen](1_3%20Rekenen.md)).

## Toewijzen

Variabelen die niet constant zijn, kunnen op een later moment opnieuw worden toegewezen met `Maak van ...`:

```
Laat x het getal 5 zijn.
  ...
Maak van x 42.
Druk x af.  ==> 42
```

Je kunt alleen een waarde toewijzen die van hetzelfde type is als waarmee de variabele is aangemaakt:

```
Laat x het getal 5 zijn.

Maak van x "Jan".
```

produceert de foutmelding `Verkeerd type: x is een getal, geen tekst.`

### Tekst toewijzen: Aanhalingstekens

Het toewijzen van een regel kan op de volgende manieren:

```
 "Hallo, wereld!"

 “Hallo, wereld!”

 „Hallo, wereld!”
```

Er is geen verschil in functionaliteit tussen de verschillende stijlen van aanhalingstekens. Je kunt ze zelfs door elkaar gebruiken! Houd er wel rekening mee dat je de juiste tekens aan het begin en het einde van de regel zet. `“` is geldig aan het begin van een regel, `”` niet (en vice-versa voor het einde van een regel).

# Datatypes

Polder is een taal met sterk gedefinieerde types. Variabelen vereisen dat deze types worden aangegeven.

Dit is de lijst van ingebouwde types die Polder kent, met een C-stijl equivalent ter vergelijking:

| C-stijl  | Polder           | Voorbeelden        |
| -------- | ---------------- | ------------------ |
| `int`    | `het getal`      | `vijf`, `5`        |
| `float`  | `het kommagetal` | `3,2`              |
| `bool`   | `de waarheid`    | `waar`, `onwaar`   |
| `string` | `de tekst`       | `"Hallo, wereld!"` |
| `char`   | `de letter`      | `'a'`              |

Let op dat je het juiste lidwoord gebruikt!

## Getalwoorden

Voor het maken van een getalvariabele kan, in plaats van het getal met cijfers op te schrijven, het woord voor dat getal worden uitgeschreven. Een paar voorbeelden:

```
Laat a het getal tweeënveertig zijn.
Laat b het getal drieëntwintig zijn.
Laat c het getal nul zijn.
Laat d het getal achthonderdachtentachtigduizendachthonderdachtentachtig zijn.

Druk a af. ==> 42
Druk b af. ==> 23
Druk c af. ==> 0
Druk d af. ==> 888888
```

- P.S.: Alleen getallen die aan elkaar geschreven zijn worden momenteel ondersteund. Een getal als `drie miljoen` wordt nog niet ondersteund.

- P.S.: Kommagetallen kunnen ook worden opgegeven met een rond getal, of een getalwoord (alhoewel in dat geval alleen hele getallen worden ondersteund. `drie komma veertien` wordt nog niet ondersteund).
