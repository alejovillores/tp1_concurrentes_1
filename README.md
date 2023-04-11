# Trabajo Práctico N° 1 - CoffeeGPT


![Rust](https://img.shields.io/badge/rust-v1.25.2-%23000000.svg?style=flat-square&logo=rust&logoColor=white)
![GitHub Actions](https://img.shields.io/badge/github%20actions-%232671E5.svg?style=flat-square&logo=githubactions&logoColor=white)


* **Autor**: [Alejo Villores](https://github.com/alejovillores)
* **Fecha** de entrega: 3/5/2023

### _Modelo y Resolución_ 

Desglozando el enunciado noto que N dispensadores que tengo por cafetera son en realidad uno por cada accion ya que por la condicion de que "Un solo dispenser a por vez puede tomar ingredientes de cada contenedor, es decir, no es posible por ejemplo que dos dispensers tomen café concurrentemente" me hace pensar que como mucho puedo tener a cada dispensador funcionando en simultaneo.

Para que la cafetera pueda procesar otro pedido, todos los dispensers deben haber terminado. Para eso, lo que se pensó es usar unas variables de condicion que espere a que todos los dispensers den ok.

Primero comienzo con una cafeteria con 4 dispensers y recurso infinito.\
El diagrama pensado fue el siguiente:
![diagrama](model.diagram.png)

Donde habrá un Thread por Dispenser y un thread por Container.

El primer dispenser en desarrollarse es el de Cafe molido, con sus contenedores de cafe molido y granos de cafe.
Luego se cosntruira el dispenser de leche y espuma. Por ultimo el de cacao y agua.

Se utilizan diferentes estructuras de sincronizacion, tales como:
 - Monitores para comunicar la cafetera con los dispensers y viceversa
 - Semaforos para dar paso al dispenser cuando hay recurso disponible
 - Barreras para que todos los dispensers antes de comenzar lean el valor que les corresponde

Hay contenedores que necesitan a su vez comunicarse con otros a la hora de rellenar sus unidades. Tambien deben saber si ese contenedor en particular quedó vacio.

Como estos contenedores que solo son se sincronizan con otro contenedor que depende de ellos, veo necesario que este contenedor sea creado por el contenedor hijo.

Por ultimo, se tiene a su vez un hilo por fuera de los dispensers que corresponde a un generador de datos estadisticos. Cada N seg imprime por stdout ciertos datos de la maquina. 


### _Test de Aceptacion_

> TODO: preguntar como hacerlos

### _Test Unitarios_ 

Para correr los test unitarios se debe ejecutar

`cargo test`

## _Extra_

Para ejecutar ``cargo fmt``, ``cargo clippy`` y ``cargo test`` todo junto, se puede ejecutar `./pre-commit.sh`
