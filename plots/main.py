import matplotlib.pyplot as plt
import json

# library -> (nice_name, color, color_with_threads)
library_name_to_info = {
    "GenedexI32Flat64": ("genedex flat64 (i32)", "blue", "cornflowerblue"),
    "GenedexU32Flat64": ("genedex flat64 (u32)", "blue", "cornflowerblue"),
    "GenedexI64Flat64": ("genedex flat64 (i64)", "blue", "cornflowerblue"),
    "GenedexI32Cond512": ("genedex cond512 (i32)", "blueviolet", "violet"),
    "GenedexU32Cond512": ("genedex cond512 (u32)", "blueviolet", "violet"),
    "GenedexI64Cond512": ("genedex cond512 (i64)", "blueviolet", "violet"),
    "Awry": ("awry", "grey", "grey"),
    "Bio": ("bio", "forestgreen", "forestgreen"),
    "FmIndex": ("fmindex", "olive", "olive"),
    "SviewFmIndexU32Vec32": ("sview vec32 (u32)", "tomato", "tomato"),
    "SviewFmIndexU32Vec128": ("sview vec128 (u64)", "orange", "orange"),
    "SviewFmIndexU64Vec32": ("sview vec32 (u32)", "tomato", "tomato"),
    "SviewFmIndexU64Vec128": ("sview vec128 (u64)", "orange", "orange"),
}

# for now, only do plots with query length 50
plot_kind_name_to_metrics = {
    "construction": ("construction_time_secs", "construction_peak_memory_usage_mb"),
    "file-io": ("write_to_file_time_secs", "read_from_file_time_secs"),
    "count": ("Count-all-50", "only_index_in_memory_size_mb"),
    "locate": ("Locate-all-50", "only_index_in_memory_size_mb"),
}

metric_to_metric_name = {
    "construction_time_secs": "running time",
    "construction_peak_memory_usage_mb": "peak memory usage",
    "write_to_file_time_secs": "write running time",
    "read_from_file_time_secs": "read running time",
    "Count-all-50": "running time",
    "Locate-all-50": "running time",
    "only_index_in_memory_size_mb": "index memory usage",
}

def metric_name_to_unit(metric_name: str):
    if metric_name.endswith("running time"):
        return "seconds"
    elif metric_name.endswith("memory usage"):
        return "megabytes"

def parse_library_config(s: str):
    parsed = s.split('-')
    return parsed[0], int(parsed[2])

def library_tuple_to_simple_name(tup):
    return library_name_to_info[tup[0]][0]

def library_tuple_to_color(tup):
    info = library_name_to_info[tup[0]]

    if tup[1] == 1:
        return info[1]
    else:
        return info[2]

def read_library_configs_and_result_data(input_texts_name: str):
    with open(f"../results/{input_texts_name}.json") as f:
        file_contents = f.read()
        results = json.loads(file_contents)

    results_list = list(results.items())
    library_configs = list(map(lambda tup: tup[0], results_list))
    results_data = list(map(lambda tup: tup[1], results_list))

    return library_configs, results_data

def extract_metric(metric: str, result: dict, input_texts_name: str):
    if metric.startswith("Locate") or metric.startswith("Count"):
        # for now only go with min running times
        search_result = result["search_metrics"].get(metric)
        if search_result:
            return search_result["min_time_secs"]
        else:
            return None
    else:
        return result[metric]

def duo_plot_for_run(plot_kind_name: str, input_texts_name: str):
    library_configs, results_data = read_library_configs_and_result_data(input_texts_name)

    left_metric, right_metric = plot_kind_name_to_metrics[plot_kind_name]

    left_metric_values = list(map(lambda result: extract_metric(left_metric, result, input_texts_name), results_data))
    right_metric_values = list(map(lambda result: extract_metric(right_metric, result, input_texts_name), results_data))
    
    left_metric_name = metric_to_metric_name[left_metric]
    left_unit = metric_name_to_unit(left_metric_name)
    right_metric_name = metric_to_metric_name[right_metric]
    right_unit = metric_name_to_unit(right_metric_name)
    
    if None in left_metric_values:
        print(f"No data yet for metric {left_metric} on input texts {input_texts_name}") 
        return
    
    if None in right_metric_values:
        print(f"No data yet for metric {right_metric} on input texts {input_texts_name}") 
        return
    
    duo_plot(
        library_configs,
        left_metric_values,
        right_metric_values,
        f"{plot_kind_name}-{input_texts_name}",
        left_metric_name,
        right_metric_name,
        left_unit,
        right_unit,
    )

def duo_plot(
        library_configs, 
        left_data, 
        right_data, 
        name,
        left_metric_name,
        right_metric_name,
        left_unit,
        right_unit,
    ):
    x = list(range(len(library_configs)))

    library_config_tuples = list(map(parse_library_config, library_configs))
    library_nice_names = list(map(library_tuple_to_simple_name , library_config_tuples))
    library_colors = list(map(library_tuple_to_color , library_config_tuples))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 7))
    
    bars1 = ax1.bar(x, left_data, color=library_colors)
    ax1.set_title(f"{left_metric_name} in {left_unit}")
    ax1.set_xticks([]) 
    ax1.bar_label(bars1)
    ax1.set_ylabel(left_unit)

    bars2 = ax2.bar(x, right_data, color=library_colors)
    ax2.set_title(f"{right_metric_name} in {right_unit}")
    ax2.set_xticks([]) 
    ax2.bar_label(bars2)
    ax2.set_ylabel(right_unit)

    fig.subplots_adjust(top=0.70)
    fig.legend(
        ax1.patches, 
        library_nice_names, 
        bbox_to_anchor=(0, 0.75, 1, 0.2),
        bbox_transform=fig.transFigure, 
        loc="lower center", 
        ncol = 4
    )

    # fig.tight_layout()
    fig.savefig(f"img/{name}.svg", bbox_inches="tight")

def all_plots_for(input_texts_name):
    for plot in ["construction", "file-io", "count", "locate"]:
        try: 
            duo_plot_for_run(plot, input_texts_name)
        except:
            print(f"An error occurred when trying to read the benchmarks JSON file for {input_texts_name}. Maybe the file is missing?")
            return

def main():
    for input_texts in ["Chromosome", "I32", "Hg38", "DoubleHg38"]:
        all_plots_for(input_texts)

if __name__ == "__main__":
    main()
