# Ingebouwde functies

**Polder** bevat een aantal ingebouwde functies die je zou verwachten van een standaardbibliotheek in andere talen.

Hieronder een lijst van alle ingebouwde functies, met een vergelijkbare C-stijl functie.

| C-stijl            | Polder         | Uitleg                                                                       | Voorbeeld                                                           |
| ------------------ | -------------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `print("Hallo\n")` | `Druk ... af.` | Schrijf een waarde naar de terminal. Maakt automatisch een nieuwe regel aan. | `Laat x de tekst "Hallo" zijn. Druk x af.`                          |
| `print("Hallo")` | `Schrijf ... op.` | Schrijf een waarde naar de terminal. Maakt geen nieuwe regel aan. | `Laat x de tekst "Hallo" zijn. Laat y de tekst ", wereld" zijn. Schrijf x op. Schrijf y op.`                          |
| `fgets(buffer)`    | `Lees ... in.` | Lees een waarde van de terminal in.                                          | `Laat naam een tekst zijn. Druk "Wat is je naam?" af. Lees naam in.` |
