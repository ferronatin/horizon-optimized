# Horizon Luz Rápida

Original Horizon-format preset based on Horizon's MIT-licensed Golden Hour.
The existing terrain pass shades its normal albedo, tint and lightmap with a
warm colour grade; the existing sky pass adds the warm horizon. No post pass,
extra colour/depth readback or intermediate post target is requested. Alpha,
lighting and the renderer's fog remain intact. This is extra shader arithmetic,
not zero GPU cost; actual FPS still needs measurement on the user's hardware.
For bloom, select Horizon Leve Bonito; Golden Hour retains the fuller example.
