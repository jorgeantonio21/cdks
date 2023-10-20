use embeddings::embeddings::Embeddings;
use std::thread::spawn;

#[tokio::main]
async fn main() {
    let sentences = vec![
        String::from("The complexity of an integrated circuit is bounded by physical limitations on the number of transistors that can be put onto one chip, the number of package terminations that can connect the processor to other parts of the system, the number of interconnections it is possible to make on the chip, and the heat that the chip can dissipate. Advancing technology makes more complex and powerful chips feasible to manufacture. A minimal hypothetical microprocessor might include only an arithmetic logic unit (ALU), and a control logic section. The ALU performs addition, subtraction, and operations such as AND or OR. Each operation of the ALU sets one or more flags in a status register, which indicate the results of the last operation (zero value, negative number, overflow, or others). The control logic retrieves instruction codes from memory and initiates the sequence of operations required for the ALU to carry out the instruction. A single operation code might affect many individual data paths, registers, and other elements of the processor. As integrated circuit technology advanced, it was feasible to manufacture more and more complex processors on a single chip. The size of data objects became larger; allowing more transistors on a chip allowed word sizes to increase from 4- and 8-bit words up to today's 64-bit words. Additional features were added to the processor architecture; more on-chip registers sped up programs, and complex instructions could be used to make more compact programs. Floating-point arithmetic, for example, was often not available on 8-bit microprocessors, but had to be carried out in software. Integration of the floating-point unit, first as a separate integrated circuit and then as part of the same microprocessor chip, sped up floating-point calculations. Occasionally, physical limitations of integrated circuits made such practices as a bit slice approach necessary. Instead of processing all of a long word on one integrated circuit, multiple circuits in parallel processed subsets of each word. While this required extra logic to handle, for example, carry and overflow within each slice, the result was a system that could handle, for example, 32-bit words using integrated circuits with a capacity for only four bits each. The ability to put large numbers of transistors on one chip makes it feasible to integrate memory on the same die as the processor. This CPU cache has the advantage of faster access than off-chip memory and increases the processing speed of the system for many applications. Processor clock frequency has increased more rapidly than external memory speed, so cache memory is necessary if the processor is not to be delayed by slower external memory."),
    ];

    let thread_join = spawn(move || {
        let embeddings =
            Embeddings::build_from_sentences(&sentences).expect("Failed to create embeddings");

        embeddings.data().to_vec()
    });

    let embeddings = thread_join.join().expect("Failed to compute embeddings");
    for embedding in embeddings {
        println!("We have computed a new embedding: \n");
        println!("Embedding: {:?}", embedding);
    }
}
