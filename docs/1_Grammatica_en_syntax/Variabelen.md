# Variabelen


## Declareren
Variabelen worden gedeclareerd met de conventie `Laat ... zijn.`

Eerst geef je de naam aan de variabele. Vervolgens wijs je een type toe. Daarna geef je de (initiele) waarde op.

Je kunt het type weglaten om deze automatisch toe te laten wijzen.

Als je wilt voorkomen dat een variabele wordt aangepast in het vervolg (De waarde is dan 'constant'), voeg dan `voorgoed` in vóór het type.

Bekijk voor de ingebouwde types [Ingebouwde_types.md](./Ingebouwde_types.md)

| C-stijl                        | Polder                                       |
| ------------------------------ | -------------------------------------------- |
| `string invoer;`               | `Laat invoer een tekst zijn.`                |
| `int x = 5;`                   | `Laat x het getal 5 zijn.`                   |
| `auto naam = "Jan";`           | `Laat naam "Jan" zijn.`                      |
| `const string taal = "Polder"` | `Laat taal voorgoed de tekst "Polder" zijn.` |

Een variabele moet óf een type, óf een initiele waarde bevatten. 


## Toewijzen
Variabelen die niet constant zijn, kunnen op een later moment opnieuw worden toegewezen met `Maak van ...`: 

```
Laat x het getal 5 zijn.
  ...
Maak van x 42.
```

Toewijzen moet met hetzelfde type als waarmee de variabele gedeclareerd is.

```
Laat x het getal 5 zijn.

Maak van x "Jan".
^ Dat is niet de bedoeling: `x` is een getal, geen tekst.
```

