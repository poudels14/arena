import matplotlib
from matplotlib.backends import backend_svg, backend_agg
from matplotlib import pyplot as plt
from matplotlib.backend_bases import _Backend
from random import randrange


@_Backend.export
class _BackendPortal(_Backend):
    FigureCanvas = backend_agg.FigureCanvas
    FigureManager = backend_agg.FigureManager

    @staticmethod
    def show(*args, **kwargs):
        print("saving file")
        plt.savefig("/home/plot-" + str(randrange(1000)) + ".png")


matplotlib.use("module://portal.matplotlib")
