import matplotlib.pyplot as plt
import json

# library -> (nice_name, color, color_with_threads)
library_name_to_info = {
    "GenedexFlat64": ("genedex flat64", "blue", "cornflowerblue"),
    "GenedexCond512": ("genedex cond512", "blueviolet", "violet"),
    "Awry": ("awry", "grey", "grey"),
    "Bio": ("bio", "forestgreen", "forestgreen"),
    "FmIndexSingle": ("fmindex single text", "olive", "olive"),
    "FmIndexMulti": ("fmindex multi text", "darkkhaki", "darkkhaki"),
    "SviewFmIndexVec32": ("sview vec32", "tomato", "tomato"),
    "SviewFmIndexVec128": ("sview vec128", "orange", "orange"),
}

# for now, only do plots with query length 50
plot_kind_name_to_metrics = {
    "Construction": ("construction_time_secs", "construction_peak_memory_usage_mb"),
    "FileIO": ("write_to_file_time_secs", "read_from_file_time_secs"),
    "Count": ("Count-all-50", "only_index_in_memory_size_mb"),
    "Locate": ("Locate-all-50", "only_index_in_memory_size_mb"),
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

extra_build_arg_to_name = {
    "LowMemory": "low memory",
}

extra_build_arg_to_pattern = {
    "LowMemory": "//",
    "none": "",
}

def metric_name_to_unit(metric_name: str):
    if metric_name.endswith("running time"):
        return "seconds"
    elif metric_name.endswith("memory usage"):
        return "gigabytes"

# parses to (library name/id, build_num_threads, extra_build_arg)
def parse_library_config(s: str):
    parsed = s.split('-')
    return parsed[0], int(parsed[2]), parsed[4]

def library_config_to_simple_name(conf):
    name = library_name_to_info[conf[0]][0]
    if conf[1] > 1:
        name += f", {conf[1]} threads"
    if conf[2] != "none":
        name += f", {extra_build_arg_to_name[conf[2]]}"
    return name

def library_config_to_color(conf):
    info = library_name_to_info[conf[0]]

    if conf[1] == 1:
        return info[1]
    else:
        return info[2]
    
def library_config_to_pattern(conf):
    return extra_build_arg_to_pattern[conf[2]]

def read_library_configs_and_result_data(input_texts_name: str):
    with open(f"../results/{input_texts_name}.json") as f:
        file_contents = f.read()
        results = json.loads(file_contents)

    results_list = sorted(results.items(), key=lambda tup: tup[0])

    library_configs = list(map(lambda tup: tup[0], results_list))
    results_data = list(map(lambda tup: tup[1], results_list))

    return library_configs, results_data

def extract_metric(metric: str, result: dict, unit: str):
    if metric.startswith("Locate") or metric.startswith("Count"):
        # for now only go with min running times
        search_result = result["search_metrics"].get(metric)
        if search_result:
            return search_result["min_time_secs"]
        else:
            return None
    else:
        value = result[metric]
        if value and unit == "gigabytes":
            return value / 1000
        else:
            return value

def duo_plot_for_run(plot_kind_name: str, input_texts_name: str):
    library_configs, results_data = read_library_configs_and_result_data(input_texts_name)
    library_config_triples = list(map(parse_library_config, library_configs))
    
    left_metric, right_metric = plot_kind_name_to_metrics[plot_kind_name]

    left_metric_name = metric_to_metric_name[left_metric]
    left_unit = metric_name_to_unit(left_metric_name)
    right_metric_name = metric_to_metric_name[right_metric]
    right_unit = metric_name_to_unit(right_metric_name)

    left_metric_values = list(map(lambda result: extract_metric(left_metric, result, left_unit), results_data))
    right_metric_values = list(map(lambda result: extract_metric(right_metric, result, right_unit), results_data))
    
    i = 0

    # to filter out different build configs for plots other than construction
    existing_libraries = set() 

    for conf, left, right in zip(library_config_triples.copy(), left_metric_values.copy(), right_metric_values.copy()):
        if left is None or right is None or (plot_kind_name != "Construction" and conf[0] in existing_libraries):
            library_config_triples.pop(i)
            left_metric_values.pop(i)
            right_metric_values.pop(i)
        else:
            i += 1
            existing_libraries.add(conf[0])
    
    if len(library_config_triples) == 0:
        return

    duo_plot(
        library_config_triples,
        left_metric_values,
        right_metric_values,
        f"{plot_kind_name}-{input_texts_name}",
        left_metric_name,
        right_metric_name,
        left_unit,
        right_unit,
    )

def duo_plot(
        library_config_triples, 
        left_data, 
        right_data, 
        name,
        left_metric_name,
        right_metric_name,
        left_unit,
        right_unit,
    ):
    x = list(range(len(library_config_triples))) 

    library_nice_names = list(map(library_config_to_simple_name, library_config_triples))
    library_colors = list(map(library_config_to_color, library_config_triples))
    library_patterns = list(map(library_config_to_pattern, library_config_triples))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 7))
    
    bar_label_fmt = "{:.2f}"

    bars1 = ax1.bar(x, left_data, color=library_colors, hatch=library_patterns)
    ax1.set_title(f"{left_metric_name} in {left_unit}")
    ax1.set_xticks([]) 
    ax1.bar_label(bars1, fmt=bar_label_fmt)
    ax1.set_ylabel(left_unit)

    bars2 = ax2.bar(x, right_data, color=library_colors, hatch=library_patterns)
    ax2.set_title(f"{right_metric_name} in {right_unit}")
    ax2.set_xticks([]) 
    ax2.bar_label(bars2, fmt=bar_label_fmt)
    ax2.set_ylabel(right_unit)

    fig.subplots_adjust(top=0.70)
    fig.legend(
        ax1.patches, 
        library_nice_names, 
        bbox_to_anchor=(0, 0.75, 1, 0.2),
        bbox_transform=fig.transFigure, 
        loc="lower center", 
        ncol = 3
    )

    fig.savefig(f"img/{name}.svg", bbox_inches="tight")

def all_plots_for(input_texts_name):
    for plot in ["Construction", "FileIO", "Count", "Locate"]:
        try: 
            duo_plot_for_run(plot, input_texts_name)
        except Exception as e:
            print(f"An error occurred when trying to generate the plots for {input_texts_name}.\n{e}")
            return

def main():
    for input_texts in ["Chromosome", "I32", "Hg38", "DoubleHg38"]:
        all_plots_for(input_texts)

if __name__ == "__main__":
    main()
