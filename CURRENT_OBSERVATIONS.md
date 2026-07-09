# Current observations

For now, I trained one model only, on Supra-50M-Instruct (full-weights finetuning). I did **230** epochs (took me **~3** hours on CPU). The loss after the **230** epochs is **0.205734** (which is normal, given the small dataset).

# Predictions with this model

## Prediction over the training dataset

I first predicted the output for `data/example_01/input.yaml`. For a small model, a few examples and a strange idea (the fact that the small model acts like a "compiler"), the results are encouraging :

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

## Prediction over a test which is not in the dataset

Then, I predicted the output for `/test/test_01/input.yaml`. The prediction is really, really worse than the one over the training dataset, but is still interesting :

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
