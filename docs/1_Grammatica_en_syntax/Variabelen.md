# Variabelen


## Declareren
Variabelen worden gedeclareerd met de conventie `Laat ... zijn.`

Eerst geef je de naam aan de variabele. Vervolgens wijs je een type toe. Daarna geef je de (initiele) waarde op.

Je kunt een variabele zonder waarde opgeven:
`Laat invoer een tekst zijn.`

Je kunt het type weglaten om deze automatisch toe te laten wijzen:
`Laat naam "Jan" zijn.`

Als je wilt voorkomen dat een variabele wordt aangepast in het vervolg (De waarde is dan 'constant'), voeg dan `voorgoed` in vóór het type.
`Laat taal voorgoed de tekst "Polder" zijn.`

Bekijk voor de ingebouwde types [Ingebouwde_types.md](./Ingebouwde_types.md)

| C-stijl                        | Polder                                       |
| ------------------------------ | -------------------------------------------- |
| `string invoer;`               | `Laat invoer een tekst zijn.`                |
| `int x = 5;`                   | `Laat x het getal 5 zijn.`                   |
| `auto naam = "Jan";`           | `Laat naam "Jan" zijn.`                      |
| `const string taal = "Polder"` | `Laat taal voorgoed de tekst "Polder" zijn.` |

Een variabele moet óf een type, óf een initiele waarde bevatten. `Laat variabele zijn.` is geen geldige zin in Polder.

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

Maak van x "Jan". ==> !! Verkeerd type: x is een getal, geen tekst.
```

