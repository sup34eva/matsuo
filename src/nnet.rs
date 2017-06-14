use std::sync::{Arc, RwLock};

use rand::random;
use rayon::prelude::*;

/// Sigmoid activation function
fn sigmoid_activation(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Random float in the -1 - 1 range
fn normal_random() -> f32 {
    2.0 * random::<f32>() - 1.0
}

static POPULATION: usize = 50;
static ELITISM: f32 = 0.2;

static RANDOM_BEHAVIOUR: f32 = 0.2;

static MUTATION_RATE: f32 = 0.1;
static MUTATION_RANGE: f32 = 0.5;
static NB_CHILDS: usize = 1;

static LEARNING_RATE: f32 = 0.01;

pub type Options = (usize, Vec<usize>, usize);

#[derive(Clone, Debug)]
struct Neuron {
	weights: Vec<f32>,
}

impl Neuron {
    fn with_population(nb: usize) -> Neuron {
        Neuron {
            weights: {
                (0..nb)
                    .into_par_iter()
                    .map(|_| normal_random())
                    .collect()
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Layer {
    neurons: Vec<Neuron>,
}

impl Layer {
    pub fn with_population(nb_neurons: usize, nb_inputs: usize) -> Layer {
        Layer {
            neurons: {
                (0..nb_neurons)
                    .into_par_iter()
                    .map(|_| {
                        Neuron::with_population(nb_inputs)
                    })
                    .collect()
            },
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FlatNetwork {
    neurons: Vec<usize>,
    weights: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
	layers: Vec<Layer>,
}

impl Network {
    pub fn into_data(self) -> FlatNetwork {
    	let mut data = FlatNetwork::default();

    	for layer in self.layers {
    		data.neurons.push(layer.neurons.len());
    		for neuron in layer.neurons {
    			for weight in neuron.weights {
    				data.weights.push(weight);
    			}
    		}
    	}

    	data
    }

    pub fn from_data(data: FlatNetwork) -> Network {
        Network {
            layers: {
                let mut index_weights = 0;
                let mut previous_neurons = 0;

                let mut res = Vec::with_capacity(data.neurons.len());
                for neuron in data.neurons {
                    let mut layer = Layer::with_population(neuron, previous_neurons);
                    for neuron in layer.neurons.iter_mut() {
                        for mut weight in neuron.weights.iter_mut() {
                            *weight = data.weights[index_weights];
                            index_weights += 1;
                        }
                    }

                    previous_neurons = neuron;
                    res.push(layer);
                }

                res
            },
        }
    }

    pub fn compute(&self, inputs: &Vec<f32>) -> Vec<f32> {
        self.layers.iter()
            .skip(1)
            .fold(
                inputs.clone(),
                |prev_layer, layer| {
                    layer.neurons.iter()
                        .map(|neuron| {
                            sigmoid_activation(
                                prev_layer.iter()
                                    .zip(
                                        neuron.weights.iter()
                                    )
                                    .map(|(prev, curr)| {
                                        prev * curr
                                    })
                                    .sum()
                            )
                        })
                        .collect()
                },
            )
    }

    fn do_run(&self, inputs: &Vec<f32>) -> Vec<Vec<f32>> {
        let mut results = Vec::new();
        results.push(inputs.to_vec());

        for (layer_index, layer) in self.layers.iter().skip(1).enumerate() {
            let prev_layer = results[layer_index].clone();
            results.push(
                layer.neurons.iter()
                    .map(|neuron| {
                        sigmoid_activation(
                            prev_layer.iter()
                                .zip(
                                    neuron.weights.iter()
                                )
                                .map(|(prev, curr)| {
                                    prev * curr
                                })
                                .sum()
                        )
                    })
                    .collect()
            );
        }

        results
    }

    pub fn train(&mut self, examples: Vec<(Vec<f32>, Vec<f32>)>) -> f32 {
        let mut prev_deltas = self.make_weights_tracker(0.0);
        let mut prev_error = -1.0;

        loop {
            let training_error_rate = {
                examples.iter()
                    .map(|&(ref inputs, ref targets)| {
                        let results = self.do_run(&inputs);
                        let weight_updates = self.calculate_weight_updates(&results, &targets);
                        self.update_weights(&weight_updates, &mut prev_deltas);
                        calculate_error(&results, &targets)
                    })
                    .sum()
            };

            if prev_error>= 0.0 && training_error_rate > prev_error {
                println!("Error rate: {}", training_error_rate);
                return training_error_rate;
            }

            prev_error = training_error_rate;
        }
    }

    fn update_weights(&mut self, updates: &Vec<Vec<Vec<f32>>>, deltas: &mut Vec<Vec<Vec<f32>>>) {
        for ((layer, updates), deltas) in self.layers.iter_mut().skip(1).zip(updates).zip(deltas.iter_mut()) {
            for ((node, updates), deltas) in layer.neurons.iter_mut().zip(updates).zip(deltas.iter_mut()) {
                for ((weight, update), delta) in node.weights.iter_mut().zip(updates).zip(deltas.iter_mut()) {
                    *delta = LEARNING_RATE * update;
                    *weight += *delta;
                }
            }
        }
    }

    fn calculate_weight_updates(&self, results: &Vec<Vec<f32>>, targets: &Vec<f32>) -> Vec<Vec<Vec<f32>>> {
        let layers = &self.layers[1..];
        let network_results = &results[1..];

        let mut network_weight_updates = Vec::new();
        let mut network_errors:Vec<Vec<f32>> = Vec::new();
        let mut next_layer_nodes: Option<&Vec<Neuron>> = None;
        for (layer_index, (layer_nodes, layer_results)) in layers.iter().zip(network_results).enumerate().rev() {
            let prev_layer_results = &results[layer_index];
            let (layer_errors, layer_weight_updates) = {
                layer_nodes.neurons.iter()
                    .zip(layer_results).enumerate()
                    .map(|(node_index, (_, &result))| {
                        let node_error = if layer_index == layers.len() - 1 {
                            result * (1.0 - result) * (targets[node_index] - result)
                        } else {
                            let sum: f32 = {
                                next_layer_nodes.expect("next_layer_nodes").iter()
                                    .zip(network_errors.last().unwrap())
                                    .map(|(next_node, &next_node_error_data)| {
                                        next_node.weights[node_index] * next_node_error_data
                                    })
                                    .sum()
                            };
                            result * (1.0 - result) * sum
                        };

                        let node_weight_updates = {
                            prev_layer_results.iter()
                                .map(|prev_layer_result| node_error * prev_layer_result)
                                .collect()
                        };

                        (node_error, node_weight_updates)
                    })
                    .unzip()
            };

            network_errors.push(layer_errors);
            network_weight_updates.push(layer_weight_updates);
            next_layer_nodes = Some(&layer_nodes.neurons);
        }

        network_weight_updates.reverse();

        network_weight_updates
    }

    fn make_weights_tracker<T: Clone>(&self, place_holder: T) -> Vec<Vec<Vec<T>>> {
        self.layers.iter()
            .skip(1)
            .map(|layer| {
                layer.neurons.iter()
                    .map(|node| {
                        node.weights.iter()
                            .map(|_| place_holder.clone())
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }
}

fn calculate_error(results: &Vec<Vec<f32>>, targets: &[f32]) -> f32 {
    let last_results = results.last().unwrap();
    let total: f32 = {
        last_results.iter()
            .zip(targets)
            .map(|(&result, &target)| {
                (target - result).powi(2)
            })
            .sum()
    };

    total / (last_results.len() as f32)
}

#[derive(Clone, Debug, Default)]
struct Genome {
    score: u32,
    network: FlatNetwork,
}

impl Genome {
    pub fn new(score: u32, network: FlatNetwork) -> Genome {
        Genome { score, network }
    }
}

#[derive(Clone, Debug, Default)]
struct Generation {
	genomes: Vec<Genome>,
}

impl Generation {
    pub fn add_genome(&mut self, genome: Genome) {
        let mut position = 0;
    	for i in 0..self.genomes.len() {
			if genome.score > self.genomes[i].score {
                position = i;
				break;
			}
    	}

    	self.genomes.insert(position, genome);
    }

    pub fn breed(&mut self, i: usize, max: usize) -> Vec<Genome> {
        let g1 = &self.genomes[i];
        let g2 = &self.genomes[max];

    	(0..NB_CHILDS)
            .into_par_iter()
            .map(|_| {
                Genome {
                    score: g1.score,
                    network: FlatNetwork {
                        neurons: g1.network.neurons.clone(),
                        weights: {
                            g1.network.weights.par_iter()
                                .zip(
                                    g2.network.weights.par_iter()
                                )
                                .map(|(w1, w2)| {
                                    let val = if random::<bool>() { *w1 } else { *w2 };
                                    if random::<f32>() <= MUTATION_RATE {
                        				val + (random::<f32>() * MUTATION_RANGE * 2.0 - MUTATION_RANGE)
                        			} else {
                                        val
                                    }
                                })
                                .collect()
                        },
                    },
                }
        	})
            .collect()
    }

    pub fn generate_next_generation(&mut self) -> Vec<FlatNetwork> {
    	let mut nexts = {
            (0..(ELITISM * POPULATION as f32).round() as usize)
                .into_par_iter()
                .map(|i| {
                    self.genomes[i].network.clone()
                })
                .chain(
                    (0..(RANDOM_BEHAVIOUR * POPULATION as f32).round() as usize)
                        .into_par_iter()
                        .map(|_| {
                            let n = &self.genomes[0].network;
                    		FlatNetwork {
                                neurons: n.neurons.clone(),
                                weights: {
                                    (0..n.weights.len())
                                        .into_par_iter()
                                        .map(|_| normal_random())
                                        .collect()
                                },
                            }
                        })
                )
                .take(POPULATION)
                .collect::<Vec<_>>()
        };

    	for max in (0..self.genomes.len()).cycle() {
    		for i in 0..max {
                let childs = self.breed(i, max);
    			for c in 0..childs.len() {
    				nexts.push(childs[c].network.clone());
    				if nexts.len() >= POPULATION {
    					return nexts;
    				}
    			}
    		}
    	}

        unreachable!()
    }
}

#[derive(Default, Debug)]
pub struct Evolver {
    options: Options,
    last_gen: Arc<RwLock<Option<Generation>>>,
}

impl Evolver {
    pub fn new(options: Options) -> Evolver {
        Evolver {
            options,
            last_gen: Arc::new(RwLock::new(None)),
        }
    }

    pub fn from_save(options: Options, networks: Vec<(FlatNetwork, u32)>) -> Evolver {
        println!("Restoring from save with {} networks", networks.len());

        let last_gen = Arc::new(RwLock::new(Some({
            let mut gen = Generation::default();

            for (network, score) in networks {
                let genome = Genome::new(score, network);
                gen.add_genome(genome);
            }

            gen
        })));

        Evolver {
            options,
            last_gen,
        }
    }

    pub fn next_generation(&self) -> Vec<Network> {
    	let networks = {
            let mut last_gen = self.last_gen.write().expect("last_gen");
            let gen = match last_gen.as_mut() {
                None => {
                    println!("Training initial generation");

                    (0..POPULATION)
                        .into_par_iter()
                        .map(|id| {
                            let (input, ref hiddens, output) = self.options;
                            let mut nn = Network {
                                layers: {
                                    let mut res = vec![
                                        Layer::with_population(input, 0),
                                    ];

                                    let mut previous_neurons = input;
                                	for hidden in hiddens {
                                		let layer = Layer::with_population(*hidden, previous_neurons);
                                		previous_neurons = *hidden;
                                		res.push(layer);
                                	}

                                	res.push(
                                        Layer::with_population(output, previous_neurons),
                                    );

                                    res
                                },
                            };

                            let mut examples = Vec::new();
                            for (l_value, l_weight) in sample_cell() {
                                for (r_value, r_weight) in sample_cell() {
                                    examples.push((
                                        vec![
                                            l_value[0], l_value[1], l_value[2],
                                            r_value[0], r_value[1], r_value[2],
                                        ],
                                        vec![(l_weight + r_weight) / 6.0],
                                    ));
                                }
                            }

                            println!("Agent {} error rate: {}", id, nn.train(examples));

                            nn.into_data()
                        })
                        .collect()
                },
                Some(gen) => {
                    gen.generate_next_generation()
                },
            };

            *last_gen = Some(Generation::default());
            gen
        };

        networks.into_par_iter()
            .map(|data| {
                Network::from_data(data)
            })
            .collect()
    }

    pub fn network_score(&self, network: Network, score: u32) {
        let genome = Genome::new(score, network.into_data());
        self.last_gen.write().expect("network_score.last_gen")
            .as_mut().expect("as_mut")
            .add_genome(genome);
    }
}

fn sample_cell() -> Vec<(Vec<f32>, f32)> {
    let mut res = Vec::new();
    res.push((
        vec![-1.0, -1.0, -1.0],
        0.0,
    ));

    for a in 0...1 {
        for b in 0...1 {
            for c in 0...1 {
                let weight = match a + b + c {
                    0 => 2.0,
                    1 => 1.0,
                    2 => 0.0,
                    3 => 3.0,
                    _ => unreachable!(),
                };

                res.push((
                    vec![
                        a as f32, b as f32, c as f32,
                    ],
                    weight,
                ));
            }
        }
    }

    res
}
