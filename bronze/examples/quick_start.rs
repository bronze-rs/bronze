use bronzeflow::prelude::*;

fn main() {
    // Create a session to submit task/workflow to scheduler
    let mut s = SessionBuilder::default().build().unwrap();

    // Create a workflow
    let wf = dag!(
        "A" => || println!("Task A") => dag!(
            "A1" => || println!("Task A1"),
            "A2" => || println!("Task A2"),
            "C1" => dag!(
                "C1" => || println!("Task C1") => dag!(
                    "C11" => || println!("Task C11")
                )
            )
        )
    )
    .build()
    .unwrap();

    // Print the dependencies tree
    wf.print_tree();

    // Submit workflow to scheduler
    s.submit("1/2 * * * * *", wf).unwrap();

    // Sleep 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
