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

The model didn't really integrate the difference between *relationship* and *emotion*, didn't really integrate the subtleties about the **character personnality**, but didn't totally hallucinate : it integrate the output format, didn't repeat himself, and proposed a pretty relevant IR.

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

As you can see, the model hallucinate : it repeated emotion|hollow_a_lot (which doesn't mean anything), gave a tiny output compared to the one of the dataset, and changed the anger, which was not pertinent. Anyway, it never was trained over it (it was totally unknown) and integrate the output format. 

# Next steps

Next things to do are :

+ train the model over a larger dataset (**~100** examples) and **~200** epochs, then test again
+ train a larger model (**~360M parameters**) with a larger dataset (**~100** examples) and **~200** epochs, then test again
+ train a LoRA instead of finetuning all the model.
+ train with an even larger datasets : **250, 500, 1500, 2500** examples.
+ evaluate the model's predictions over many tests
+ integrate the IR in the engine

---

# Supra 50 M Instruct (second time, small dataset)

The model is available [here](https://huggingface.co/CogStateIR).

## Training setup

- Base model : https://huggingface.co/SupraLabs/Supra-50M-Instruct
- Parameters : 51.8M
- Dataset : 135 examples
- Epochs : 100
- Learning rate : 0.0001
- Hardware : Apple M2 Pro on GPU
- Duration : ~20 minutes (thanks to Metal)

## Results

Loss after training : 0.397188.
Validation loss after training: 1.135684.

## Prediction - training example

Model's raw output :
```text
"emotion|anger|increases_a_little\nemotion|hurt|increases\nrelationship|closeness|increases\nrelationship|trust|increases_a_little\nbelief|user_is_dismissive|increases"
```
Model's prediction :
```yaml
state_changes:
  emotion:
    anger: increases_a_little
    hurt: increases
  relationship:
    closeness: increases
    trust: increases_a_little
  belief:
    user_is_dismissive: increases
  memory: null
  reflection: null
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

The model didn't really integrate the difference between *relationship* and *emotion*, didn't really integrate the subtleties about the **character personnality**, but didn't totally hallucinate : it integrate the output format, didn't repeat himself, and proposed a pretty relevant IR (not totally absurd). The result is quite the same as for the first [model](#supra-50-m-instruct), but a little better (more content).


## Prediction - held-out example

Model's raw output :

```text
RAW OUTPUT:
"emotion|anger|increases\nemotion|fear|increases\nemotion|fear|increases\nrelationship|fear|increases\nrelationship|fear|increases\nrelationship|fear|increases\nrelationship|trust|increases\nbelief|user_cant_take|increases\nreflection|required"
```
Model's output :

```yaml
state_changes:
  emotion:
    anger: increases
    fear: increases
  relationship:
    fear: increases
    trust: increases
  belief:
    user_cant_take: increases
  memory: null
  reflection: required
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

As you can see, the modeld didn't hallucinate this time : it didn't repeated himself, gave a substancial output compared to the one of the first [model](#prediction---held-out-example). We still have the problem of the pertinence of the changes. I think this can be because of :

- the small size of the model
- the small size of the dataset 

# Next steps

Next things to do are :

+ train a larger model (**~360M parameters**) with a larger dataset (**~500** examples) and **~100** epochs (to avoid overtraining and memorization)
+ train a LoRA instead of finetuning all the model.
+ train with an even larger datasets : **250, 500, 1500, 2500** examples.
+ evaluate the model's predictions over many tests
+ integrate the IR in the engine

---

# Supra 50 M Instruct (third time, medium dataset)

The model is available [here](https://huggingface.co/CogStateIR).

## Training setup

- Base model : https://huggingface.co/SupraLabs/Supra-50M-Instruct
- Parameters : 51.8M
- Dataset : 315 examples
- Epochs : 100
- Learning rate : 0.0001
- Hardware : Apple M2 Pro on GPU
- Duration : ~1h (thanks to Metal)

## Results

Loss after training :  0.393566.
Validation loss after training: 0.832417.

## Prediction - training example

Model's raw output :
```text
"emotion|anger|increases_a_lot\nemotion|hurt|increases_a_little\nrelationship|closeness|increases_a_little\nbelief|user_is_frustrated|increases_a_little"
```
Model's prediction :
```yaml
state_changes:
  emotion:
    anger: increases_a_lot
    hurt: increases_a_little
  relationship:
    closeness: increases_a_little
  belief:
    user_is_frustrated: increases_a_little
  memory: null
  reflection: null
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

The model didn't really integrate the difference between *relationship* and *emotion*, didn't really integrate the subtleties about the **character personnality**, but didn't totally hallucinate : it integrate the output format, didn't repeat himself, and proposed a pretty relevant IR (not totally absurd). The result is quite the same as for the first [model](#supra-50-m-instruct), but a little better (more content).


## Prediction - held-out example

Model's raw output :

```text
RAW OUTPUT:
"emotion|anger|increases\nemotion|hope|increases\nemotion|hope|increases\nrelationship|trust|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelationship|trust_is_frustration|increases\nrelations"
```
Model's output :

```yaml
state_changes:
  emotion:
    anger: increases
    hope: increases
  relationship:
    trust: increases
    trust_is_frustration: increases
  belief: null
  memory: null
  reflection: null
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

The problem is that here, the model seems worst than the one with 135 examples... so I decided to run other tests :


## Prediction - held-out example 2

RAW OUTPUT:
"no_changes"

Model's output :
```yaml
state_changes: no_changes
```
 
Output of the dataset :

```yaml
state_changes:
  emotion:
    contentment: increases_a_little
```

This time, the model didn't hallucinate. It only decided that no changes were required, which is not absurd.

> ![NOTE]
> This examples and the following ones are from the dataset, but the model wasn't trained on these.

## Prediction - held-out example 3

RAW OUTPUT:
"emotion|contentment|increases_a_little\nemotion|warmth|increases_a_little\nbelief|increases_a_little\nbelief|increases_a_little\nbelief|increases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a_little\nbelief|decreases_a"

Model's output :

```yaml
state_changes:
  emotion:
    contentment: increases_a_little
    warmth: increases_a_little
  relationship: null
  belief: null
  memory: null
  reflection: null
```

Output from the dataset :

```yaml
state_changes:
  emotion:
    contentment: increases_a_little
```

This time, the result is very good. While the model wasn't trained over this examples (he didn't know this one), its prediction is adequate : he predicted `contentment`, but also added warmth, which is not absurd.
We still have a problem : the hallucination. The model repeated himself. Anyway, thanks to the internal parser, this doesn't affect the final output.

## Prediction - held-out example 4

RAW OUTPUT:
"no_changes"

Output of the model :

```yaml
state_changes: no_changes
```

Output of the dataset :

```yaml
state_changes: no_changes
```

While the model wasn't trained over this example, it prediction is correct. He 'understands' that there are no changes or minor changes in basic conversations.

## Prediction - held-out example 5

RAW OUTPUT:
"emotion|determination|increases\nemotion|determination|increases\nemotion|determination|increases\nemotion|determination|increases\nrelationship|determination|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nbelief|exhaustion|increases\nreflection|required"

Model's output :
```yaml
state_changes:
  emotion:
    determination: increases
  relationship:
    determination: increases
  belief:
    exhaustion: increases
  memory: null
  reflection: required
```

Output of the dataset :
```yaml
state_changes:
  emotion:
    exhaustion: increases_a_little
    relief: increases
    warmth: increases_a_little
  relationship:
    closeness: increases_a_little
```

This examples is more complex. It manages to predict something coherent (exhaustion, determination, null memory is right...), but not totally right : exhaustion is NOT a belief, reflection wasn't really required, and closeness was required.

## Current model's behaviour :

|  Situation type            | Behaviour                                                          |
| ---------------------------- | ----------------------------------------------------------------------------- |
| Very simple                  | Almost always correct                                                      |
| Casual conversation         | Very good, sometimes even better than the dataset (`warmth`)   |
| Situation complex with emotions | Understands concepts but makes magnitude and classification errors |
| Situation very complex    | Begins mixing classes, forgets elements, hallucinate more      |

# Next steps

I think the real bottleneck is the model size : a 360M parameters could be more appropriate. Also, a larger dataset would make the model understand more situations and more complex ones.

Next things to do are :

+ train a larger model (**~360M parameters**) with a larger dataset / the same one (**~500** examples) and **~100** epochs (to avoid overtraining and memorization)
+ train a LoRA instead of finetuning all the model.
+ train with an even  datasets : **500, 1500, 2500** examples.
+ evaluate the model's predictions over many tests
+ integrate the IR in the engine