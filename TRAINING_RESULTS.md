# Supra 50 M Instruct

The model is available [here](https://huggingface.co/CogStateIR).
## Training setup

- Base model : https://huggingface.co/SupraLabs/Supra-50M-Instruct
- Parameters : 51.8M
- Dataset : 11 examples
- Epochs : 230
- Learning rate : 0.0001
- Hardware : Apple M2 Pro
- Duration : ~3h

## Results

Loss after training : 0.205734.

## Prediction - training example

Model's raw output :
```text
"emotion|anger|increases_a_little\nrelationship|trust|increases_a_little\nmemory|reinforce_previous_conflict\nreflection|required"
```
Model's prediction :
```yaml
state_changes:
  emotion:
    anger: increases_a_little
  relationship:
    trust: increases_a_little
  memory: reinforce_previous_conflict
  reflection: required
```

Output of the dataset :

```yaml
state_changes:
  relationship:
    trust: increases_a_little
  emotion:
    anger: decreases
  belief:
    people_can_change: increases_a_little
  memory: reinforce_previous_conflict
  reflection: required
```

The model didn't really integrated the difference between *relationship* and *emotion*, didn't really integrated the subtleties about the **character personnality**, but didn't totally hallucinated : he integrated the output format, didn't repeat himself, and proposed a pretty relevant IR.

## Prediction - held-out example

Model's raw output :

```text
RAW OUTPUT:
"emotion|anger|increases_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot\nemotion|hollow_a_lot"
```
Model's output :

```yaml
state_changes:
  emotion:
    anger: increases_a_lot
```

Output of the test dataset :

```yaml
state_changes:
  relationship:
    respect: increases_a_little
    disappointment: decreases
  emotion:
    irritation: decreases
    pride: increases_a_little
  belief:
    effort_matters: increases
  memory: reinforce_previous_conflict
  reflection: required
```

As you can see, the model hallucinated : he repeated emotion|hollow_a_lot (which doesn't mean anything), gave a tiny output compared to the one of the dataset, and changed the anger, which was not pertinent. Anyway, he never was trained over it (it was totally unknown) and integrated the output format. 

# Next steps

Next things to do are :

+ train the model over a larger dataset (**~100** examples) and **~200** epochs, then test again
+ train a larger model (**~360M parameters**) with a larger dataset (**~100** examples) and **~200** epochs, then test again
+ train a LoRA instead of finetuning all the model.
+ train with an even largers datasets : **250, 500, 1500, 2500** examples.
+ evaluate the model's predictions over many tests
+ integrate the IR in the engine