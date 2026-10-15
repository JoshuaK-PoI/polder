# Ingebouwde types

Polder is een taal met sterk gedefinieerde types. Variabelen en functies vereisen dat deze types worden aangegeven.

Dit is de lijst van ingebouwde types die Polder kent, met een C-stijl equivalent ter vergelijking:

| C-stijl  | Polder       | Voorbeelden              |
| -------- | ------------ | ------------------------ |
| `int`    | `getal`      | `vijf`, `5`              |
| `float`  | `kommagetal` | `3,2`                    |
| `bool`   | `waarheid`   | `waar`, `onwaar`         |
| `string` | `regel`      | `"Hallo, wereld!"`       |
| `char`   | `letter`     | `'a'`                    |

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

## Regels

Het toewijzen van een regel kan op de volgende manieren:

```
 "Hallo, wereld!"

 “Hallo, wereld!”

 „Hallo, wereld!”
 ```

Er is geen verschil in functionaliteit tussen de verschillende stijlen van aanhalingstekens. Je kunt ze zelfs door elkaar gebruiken! Houd er wel rekening mee dat je de juiste tekens aan het begin en het einde van de regel zet. `“` is geldig aan het begin van een regel, `”` niet (en vice-versa voor het einde van een regel).
